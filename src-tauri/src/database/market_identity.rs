use rusqlite::{params, Connection};

const REPAIR_EVIDENCE: &str = "dataset_name+source_path+legacy_asset_prefix";

pub fn normalize_asset_identity(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err("ativo é obrigatório".into());
    }
    if normalized.len() > 32
        || !normalized
            .chars()
            .any(|character| character.is_ascii_alphanumeric())
        || !normalized
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".-^=/:".contains(character))
    {
        return Err(
            "ativo inválido: informe somente o ticker/identificador de mercado, por exemplo AAPL"
                .into(),
        );
    }
    Ok(normalized)
}

pub fn validate_same_market_identity(
    development_asset: &str,
    development_timeframe: &str,
    oos_asset: &str,
    oos_timeframe: &str,
) -> Result<(), String> {
    if development_asset != oos_asset || development_timeframe != oos_timeframe {
        return Err("DEV e OOS devem usar o mesmo ativo e timeframe".into());
    }
    Ok(())
}

fn leading_asset_candidate(value: &str) -> Option<String> {
    let filename = value.rsplit(['/', '\\']).next()?.trim();
    let candidate = filename.split('_').next()?.trim();
    normalize_asset_identity(candidate).ok()
}

fn legacy_asset_repair_candidate(name: &str, source_path: &str, asset: &str) -> Option<String> {
    let name_candidate = leading_asset_candidate(name)?;
    let source_candidate = leading_asset_candidate(source_path)?;
    if name_candidate != source_candidate {
        return None;
    }

    let legacy_asset = asset.trim().to_ascii_uppercase();
    let suffix = legacy_asset.strip_prefix(&name_candidate)?;
    if suffix.starts_with(" - ") || suffix.starts_with("- ") {
        Some(name_candidate)
    } else {
        None
    }
}

pub fn repair_legacy_asset_identities(connection: &mut Connection) -> Result<usize, String> {
    let candidates = {
        let mut statement = connection
            .prepare(
                "SELECT d.id,d.name,d.asset,d.source_path
                 FROM market_datasets d
                 WHERE NOT EXISTS(
                   SELECT 1 FROM market_dataset_identity_repairs r WHERE r.dataset_id=d.id
                 )",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        rows
    };

    let repairs = candidates
        .into_iter()
        .filter_map(|(id, name, asset, source_path)| {
            legacy_asset_repair_candidate(&name, &source_path, &asset)
                .map(|normalized| (id, asset, normalized))
        })
        .collect::<Vec<_>>();

    if repairs.is_empty() {
        return Ok(0);
    }

    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let mut repaired = 0;
    for (dataset_id, previous_asset, normalized_asset) in repairs {
        let changed = transaction
            .execute(
                "UPDATE market_datasets SET asset=?1 WHERE id=?2 AND asset=?3",
                params![normalized_asset, dataset_id, previous_asset],
            )
            .map_err(|error| error.to_string())?;
        if changed == 0 {
            continue;
        }
        transaction
            .execute(
                "UPDATE market_experiments SET asset=?1
                 WHERE dataset_id=?2 AND asset=?3",
                params![normalized_asset, dataset_id, previous_asset],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "INSERT INTO market_dataset_identity_repairs(
                   dataset_id,previous_asset,normalized_asset,evidence
                 ) VALUES(?1,?2,?3,?4)",
                params![
                    dataset_id,
                    previous_asset,
                    normalized_asset,
                    REPAIR_EVIDENCE
                ],
            )
            .map_err(|error| error.to_string())?;
        repaired += 1;
    }
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(repaired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{self, market_models::ImportMarketDatasetInput, market_repository};

    #[test]
    fn normalizes_canonical_assets_and_rejects_descriptions() {
        assert_eq!(normalize_asset_identity(" aapl ").unwrap(), "AAPL");
        assert_eq!(normalize_asset_identity("brk.b").unwrap(), "BRK.B");
        assert_eq!(normalize_asset_identity("btc-usd").unwrap(), "BTC-USD");
        assert!(normalize_asset_identity("AAPL - REAL").is_err());
        assert!(normalize_asset_identity("AAPL - 20").is_err());
    }

    #[test]
    fn dataset_import_rejects_descriptive_asset_metadata_before_reading_the_file() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        let error = market_repository::import_dataset(
            &mut connection,
            &ImportMarketDatasetInput {
                path: "missing.csv".into(),
                name: "AAPL_real_15m_OOS".into(),
                asset: "AAPL - REAL".into(),
                timeframe: "15M".into(),
                currency: Some("USD".into()),
                market: Some("US_EQUITIES".into()),
                timezone: Some("America/New_York".into()),
                session_type: Some("REGULAR".into()),
            },
        )
        .unwrap_err();
        assert!(error.contains("ticker/identificador de mercado"));
    }

    #[test]
    fn validates_dev_and_oos_only_by_asset_and_timeframe() {
        assert!(validate_same_market_identity("AAPL", "15M", "AAPL", "15M").is_ok());
        assert!(validate_same_market_identity("AAPL", "15M", "MSFT", "15M").is_err());
        assert!(validate_same_market_identity("AAPL", "15M", "AAPL", "1D").is_err());

        // Dataset names are deliberately absent from this contract: different names with the
        // same market identity are accepted, and similar names cannot mask different assets.
        let differently_named_dev = ("AAPL", "15M");
        let differently_named_oos = ("AAPL", "15M");
        assert!(validate_same_market_identity(
            differently_named_dev.0,
            differently_named_dev.1,
            differently_named_oos.0,
            differently_named_oos.1
        )
        .is_ok());
        assert!(validate_same_market_identity("AAPL", "15M", "AAPLX", "15M").is_err());
    }

    #[test]
    fn repairs_only_unambiguous_legacy_metadata_and_preserves_market_data() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        for (id, name, asset, source, fingerprint) in [
            (
                "dataset-dev",
                "AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated",
                "AAPL - 20",
                "C:\\Downloads\\AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated.csv",
                "fingerprint-dev",
            ),
            (
                "dataset-oos",
                "AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated",
                "AAPL - REAL",
                "C:\\Downloads\\AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated.csv",
                "fingerprint-oos",
            ),
            (
                "dataset-ambiguous",
                "AAPL_real_1D_2022_treated",
                "AAPL3",
                "C:\\Downloads\\AAPL_real_1D_2022_treated.csv",
                "fingerprint-ambiguous",
            ),
        ] {
            connection.execute(
                "INSERT INTO market_datasets(id,name,asset,timeframe,currency,start_at,end_at,candle_count,fingerprint,source_path)
                 VALUES(?1,?2,?3,'15M','USD','2026-01-01','2026-01-02',1,?4,?5)",
                params![id, name, asset, fingerprint, source],
            ).unwrap();
            connection.execute(
                "INSERT INTO market_candles(dataset_id,candle_index,timestamp,open,high,low,close,volume)
                 VALUES(?1,0,'2026-01-01',10,11,9,10,100)",
                [id],
            ).unwrap();
            connection.execute(
                "INSERT INTO market_experiments(id,name,dataset_id,asset,timeframe,currency,initial_capital,risk_profile_id,random_seed,fee_pct,slippage_pct,config_json,status)
                 VALUES(?1,?2,?3,?4,'15M','USD',10000,'balanced-v1',42,0.1,0.05,'{}','completed')",
                params![format!("experiment-{id}"), name, id, asset],
            ).unwrap();
        }

        assert_eq!(repair_legacy_asset_identities(&mut connection).unwrap(), 2);
        assert_eq!(repair_legacy_asset_identities(&mut connection).unwrap(), 0);

        for id in ["dataset-dev", "dataset-oos"] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT asset FROM market_datasets WHERE id=?1",
                        [id],
                        |row| row.get::<_, String>(0),
                    )
                    .unwrap(),
                "AAPL"
            );
            assert_eq!(
                connection
                    .query_row(
                        "SELECT asset FROM market_experiments WHERE dataset_id=?1",
                        [id],
                        |row| row.get::<_, String>(0),
                    )
                    .unwrap(),
                "AAPL"
            );
        }
        assert_eq!(
            connection
                .query_row(
                    "SELECT asset FROM market_datasets WHERE id='dataset-ambiguous'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "AAPL3"
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM market_candles", [], |row| {
                    row.get::<_, usize>(0)
                })
                .unwrap(),
            3
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_dataset_identity_repairs",
                    [],
                    |row| row.get::<_, usize>(0),
                )
                .unwrap(),
            2
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT fingerprint FROM market_datasets WHERE id='dataset-dev'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "fingerprint-dev"
        );
    }
}
