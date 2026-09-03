use super::engineering_models::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;

pub fn get_calibration(connection: &Connection) -> Result<EngineeringCalibration, String> {
    connection
        .query_row(
            "SELECT pinch_start_threshold,pinch_release_threshold,smoothing_alpha,rotation_sensitivity,min_scale,max_scale,comfortable_hand_distance,calibrated,updated_at FROM engineering_calibration WHERE id=1",
            [],
            |row| Ok(EngineeringCalibration {
                pinch_start_threshold: row.get(0)?,
                pinch_release_threshold: row.get(1)?,
                smoothing_alpha: row.get(2)?,
                rotation_sensitivity: row.get(3)?,
                min_scale: row.get(4)?,
                max_scale: row.get(5)?,
                comfortable_hand_distance: row.get(6)?,
                calibrated: row.get(7)?,
                updated_at: row.get(8)?,
            }),
        )
        .map_err(|error| error.to_string())
}

pub fn update_calibration(
    connection: &Connection,
    input: &EngineeringCalibrationInput,
) -> Result<EngineeringCalibration, String> {
    validate(input)?;
    connection
        .execute(
            "UPDATE engineering_calibration SET pinch_start_threshold=?1,pinch_release_threshold=?2,smoothing_alpha=?3,rotation_sensitivity=?4,min_scale=?5,max_scale=?6,comfortable_hand_distance=?7,calibrated=?8,updated_at=CURRENT_TIMESTAMP WHERE id=1",
            params![input.pinch_start_threshold, input.pinch_release_threshold, input.smoothing_alpha, input.rotation_sensitivity, input.min_scale, input.max_scale, input.comfortable_hand_distance, input.calibrated],
        )
        .map_err(|error| error.to_string())?;
    get_calibration(connection)
}

pub fn reset_calibration(connection: &Connection) -> Result<EngineeringCalibration, String> {
    connection
        .execute(
            "UPDATE engineering_calibration SET pinch_start_threshold=0.045,pinch_release_threshold=0.065,smoothing_alpha=0.38,rotation_sensitivity=3.2,min_scale=0.45,max_scale=2.4,comfortable_hand_distance=0.42,calibrated=0,updated_at=CURRENT_TIMESTAMP WHERE id=1",
            [],
        )
        .map_err(|error| error.to_string())?;
    get_calibration(connection)
}

fn validate(input: &EngineeringCalibrationInput) -> Result<(), String> {
    let finite = [
        input.pinch_start_threshold,
        input.pinch_release_threshold,
        input.smoothing_alpha,
        input.rotation_sensitivity,
        input.min_scale,
        input.max_scale,
        input.comfortable_hand_distance,
    ]
    .iter()
    .all(|value| value.is_finite());
    if !finite {
        return Err("A calibração contém valor não finito".into());
    }
    if !(0.015..=0.100).contains(&input.pinch_start_threshold) {
        return Err("Pinch start deve estar entre 0.015 e 0.100".into());
    }
    if !(0.025..=0.140).contains(&input.pinch_release_threshold)
        || input.pinch_release_threshold <= input.pinch_start_threshold
    {
        return Err("Pinch release deve ser maior que pinch start e no máximo 0.140".into());
    }
    if !(0.05..=1.0).contains(&input.smoothing_alpha) {
        return Err("Smoothing deve estar entre 0.05 e 1.0".into());
    }
    if !(0.25..=8.0).contains(&input.rotation_sensitivity) {
        return Err("Sensibilidade de rotação deve estar entre 0.25 e 8.0".into());
    }
    if !(0.20..=1.0).contains(&input.min_scale)
        || !(1.0..=5.0).contains(&input.max_scale)
        || input.max_scale <= input.min_scale
    {
        return Err("Limites de escala inválidos".into());
    }
    if !(0.05..=1.0).contains(&input.comfortable_hand_distance) {
        return Err("Distância confortável deve estar entre 0.05 e 1.0".into());
    }
    Ok(())
}

const RELATIONSHIP_TYPES: [&str; 8] = [
    "connected_to",
    "contains",
    "supports",
    "drives",
    "mounted_on",
    "adjacent_to",
    "depends_on",
    "custom",
];

fn require_text(value: &str, label: &str, max: usize) -> Result<(), String> {
    let size = value.trim().chars().count();
    if size == 0 || size > max {
        return Err(format!("{label} deve conter entre 1 e {max} caracteres"));
    }
    Ok(())
}

pub fn register_model(
    connection: &mut Connection,
    input: &RegisterEngineeringModelInput,
) -> Result<AssemblyIntelligenceSnapshot, String> {
    require_text(&input.model_identity, "Identidade do modelo", 128)?;
    require_text(&input.file_name, "Nome do modelo", 260)?;
    if !matches!(input.format.as_str(), "GLB" | "GLTF") || input.byte_size <= 0 {
        return Err("Metadados do modelo inválidos".into());
    }
    if input.components.is_empty() || input.components.len() > 50_000 {
        return Err("Quantidade de componentes inválida".into());
    }
    let mut identities = HashSet::new();
    for component in &input.components {
        require_text(
            &component.component_identity,
            "Identidade do componente",
            1000,
        )?;
        require_text(&component.original_name, "Nome original", 500)?;
        require_text(&component.structural_path, "Caminho estrutural", 4000)?;
        if !identities.insert(&component.component_identity) {
            return Err("O modelo contém identidades de componente duplicadas".into());
        }
    }
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction.execute(
        "INSERT INTO engineering_models(model_identity,file_name,format,byte_size) VALUES (?1,?2,?3,?4) ON CONFLICT(model_identity) DO UPDATE SET file_name=excluded.file_name,format=excluded.format,byte_size=excluded.byte_size,updated_at=CURRENT_TIMESTAMP",
        params![input.model_identity,input.file_name,input.format,input.byte_size],
    ).map_err(|error| error.to_string())?;
    for component in &input.components {
        transaction.execute(
            "INSERT INTO engineering_model_components(model_identity,component_identity,original_name,structural_path,component_type,selectable) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(model_identity,component_identity) DO UPDATE SET original_name=excluded.original_name,structural_path=excluded.structural_path,component_type=excluded.component_type,selectable=excluded.selectable,last_seen_at=CURRENT_TIMESTAMP",
            params![input.model_identity,component.component_identity,component.original_name,component.structural_path,component.component_type,component.selectable],
        ).map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())?;
    get_snapshot(connection, &input.model_identity)
}

pub fn get_snapshot(
    connection: &Connection,
    model_identity: &str,
) -> Result<AssemblyIntelligenceSnapshot, String> {
    require_text(model_identity, "Identidade do modelo", 128)?;
    let mut semantics_statement = connection.prepare("SELECT s.model_identity,s.component_identity,c.original_name,c.structural_path,c.component_type,s.semantic_label,s.subsystem_id,s.role,s.description,s.notes,s.created_at,s.updated_at FROM engineering_component_semantics s JOIN engineering_model_components c ON c.model_identity=s.model_identity AND c.component_identity=s.component_identity WHERE s.model_identity=?1 ORDER BY s.component_identity").map_err(|e| e.to_string())?;
    let semantics = semantics_statement
        .query_map([model_identity], |row| {
            Ok(ComponentSemantic {
                model_identity: row.get(0)?,
                component_identity: row.get(1)?,
                original_name: row.get(2)?,
                structural_path: row.get(3)?,
                component_type: row.get(4)?,
                semantic_label: row.get(5)?,
                subsystem_id: row.get(6)?,
                role: row.get(7)?,
                description: row.get(8)?,
                notes: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut subsystem_statement = connection.prepare("SELECT id,model_identity,name,description,parent_subsystem_id,created_at,updated_at FROM engineering_subsystems WHERE model_identity=?1 ORDER BY name COLLATE NOCASE").map_err(|e| e.to_string())?;
    let subsystems = subsystem_statement
        .query_map([model_identity], |row| {
            Ok(EngineeringSubsystem {
                id: row.get(0)?,
                model_identity: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                parent_subsystem_id: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut relationship_statement = connection.prepare("SELECT id,model_identity,source_component_identity,target_component_identity,relationship_type,description,created_at,updated_at FROM engineering_component_relationships WHERE model_identity=?1 ORDER BY created_at,id").map_err(|e| e.to_string())?;
    let relationships = relationship_statement
        .query_map([model_identity], |row| {
            Ok(ComponentRelationship {
                id: row.get(0)?,
                model_identity: row.get(1)?,
                source_component_identity: row.get(2)?,
                target_component_identity: row.get(3)?,
                relationship_type: row.get(4)?,
                description: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(AssemblyIntelligenceSnapshot {
        semantics,
        subsystems,
        relationships,
    })
}

pub fn save_semantic(
    connection: &Connection,
    input: &ComponentSemanticInput,
) -> Result<ComponentSemantic, String> {
    require_text(&input.model_identity, "Identidade do modelo", 128)?;
    require_text(&input.component_identity, "Identidade do componente", 1000)?;
    for (value, label, max) in [
        (&input.semantic_label, "Rótulo", 200),
        (&input.role, "Função", 200),
        (&input.description, "Descrição", 4000),
        (&input.notes, "Notas", 8000),
    ] {
        if value.chars().count() > max {
            return Err(format!("{label} excede {max} caracteres"));
        }
    }
    if let Some(subsystem_id) = &input.subsystem_id {
        let valid = connection.query_row("SELECT EXISTS(SELECT 1 FROM engineering_subsystems WHERE id=?1 AND model_identity=?2)", params![subsystem_id,input.model_identity], |row| row.get::<_, bool>(0)).map_err(|e| e.to_string())?;
        if !valid {
            return Err("O subsistema não pertence ao modelo atual".into());
        }
    }
    connection.execute("INSERT INTO engineering_component_semantics(model_identity,component_identity,semantic_label,subsystem_id,role,description,notes) VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(model_identity,component_identity) DO UPDATE SET semantic_label=excluded.semantic_label,subsystem_id=excluded.subsystem_id,role=excluded.role,description=excluded.description,notes=excluded.notes,updated_at=CURRENT_TIMESTAMP", params![input.model_identity,input.component_identity,input.semantic_label.trim(),input.subsystem_id,input.role.trim(),input.description.trim(),input.notes.trim()]).map_err(|e| e.to_string())?;
    get_snapshot(connection, &input.model_identity)?
        .semantics
        .into_iter()
        .find(|item| item.component_identity == input.component_identity)
        .ok_or_else(|| "Semântica não encontrada após salvar".into())
}

pub fn save_subsystem(
    connection: &Connection,
    input: &EngineeringSubsystemInput,
) -> Result<EngineeringSubsystem, String> {
    require_text(&input.id, "ID do subsistema", 128)?;
    require_text(&input.model_identity, "Identidade do modelo", 128)?;
    require_text(&input.name, "Nome", 160)?;
    if input.description.chars().count() > 2000 {
        return Err("Descrição excede 2000 caracteres".into());
    }
    if input.parent_subsystem_id.as_deref() == Some(input.id.as_str()) {
        return Err("Um subsistema não pode ser pai de si mesmo".into());
    }
    let existing_model = connection
        .query_row(
            "SELECT model_identity FROM engineering_subsystems WHERE id=?1",
            [&input.id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if existing_model
        .as_deref()
        .is_some_and(|identity| identity != input.model_identity)
    {
        return Err("O subsistema pertence a outro modelo".into());
    }
    if let Some(parent_id) = &input.parent_subsystem_id {
        let valid_parent = connection.query_row("SELECT EXISTS(SELECT 1 FROM engineering_subsystems WHERE id=?1 AND model_identity=?2)", params![parent_id,input.model_identity], |row| row.get::<_, bool>(0)).map_err(|e| e.to_string())?;
        if !valid_parent {
            return Err("O subsistema pai não pertence ao modelo atual".into());
        }
        let creates_cycle = connection.query_row("WITH RECURSIVE descendants(id) AS (SELECT id FROM engineering_subsystems WHERE parent_subsystem_id=?1 UNION ALL SELECT s.id FROM engineering_subsystems s JOIN descendants d ON s.parent_subsystem_id=d.id) SELECT EXISTS(SELECT 1 FROM descendants WHERE id=?2)", params![input.id,parent_id], |row| row.get::<_, bool>(0)).map_err(|e| e.to_string())?;
        if creates_cycle {
            return Err("A hierarquia de subsistemas formaria um ciclo".into());
        }
    }
    connection.execute("INSERT INTO engineering_subsystems(id,model_identity,name,description,parent_subsystem_id) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET name=excluded.name,description=excluded.description,parent_subsystem_id=excluded.parent_subsystem_id,updated_at=CURRENT_TIMESTAMP",params![input.id,input.model_identity,input.name.trim(),input.description.trim(),input.parent_subsystem_id]).map_err(|e| e.to_string())?;
    get_snapshot(connection, &input.model_identity)?
        .subsystems
        .into_iter()
        .find(|item| item.id == input.id)
        .ok_or_else(|| "Subsistema não encontrado após salvar".into())
}

pub fn delete_subsystem(
    connection: &Connection,
    model_identity: &str,
    id: &str,
) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM engineering_subsystems WHERE id=?1 AND model_identity=?2",
            params![id, model_identity],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn save_relationship(
    connection: &Connection,
    input: &ComponentRelationshipInput,
) -> Result<ComponentRelationship, String> {
    require_text(&input.id, "ID da relação", 128)?;
    require_text(&input.model_identity, "Identidade do modelo", 128)?;
    if input.source_component_identity == input.target_component_identity {
        return Err("Uma relação não pode apontar para o mesmo componente".into());
    }
    if !RELATIONSHIP_TYPES.contains(&input.relationship_type.as_str()) {
        return Err("Tipo de relação inválido".into());
    }
    connection.execute("INSERT INTO engineering_component_relationships(id,model_identity,source_component_identity,target_component_identity,relationship_type,description) VALUES (?1,?2,?3,?4,?5,?6)",params![input.id,input.model_identity,input.source_component_identity,input.target_component_identity,input.relationship_type,input.description.trim()]).map_err(|e| e.to_string())?;
    get_snapshot(connection, &input.model_identity)?
        .relationships
        .into_iter()
        .find(|item| item.id == input.id)
        .ok_or_else(|| "Relação não encontrada após salvar".into())
}

pub fn delete_relationship(
    connection: &Connection,
    model_identity: &str,
    id: &str,
) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM engineering_component_relationships WHERE id=?1 AND model_identity=?2",
            params![id, model_identity],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn input() -> EngineeringCalibrationInput {
        EngineeringCalibrationInput {
            pinch_start_threshold: 0.04,
            pinch_release_threshold: 0.07,
            smoothing_alpha: 0.4,
            rotation_sensitivity: 3.5,
            min_scale: 0.5,
            max_scale: 2.8,
            comfortable_hand_distance: 0.48,
            calibrated: true,
        }
    }

    #[test]
    fn calibration_is_persistent_and_resettable() {
        let path =
            std::env::temp_dir().join(format!("azriel-engineering-{}.db", std::process::id()));
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            let saved = update_calibration(&connection, &input()).unwrap();
            assert!(saved.calibrated);
            assert_eq!(saved.rotation_sensitivity, 3.5);
        }
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            assert_eq!(
                get_calibration(&connection)
                    .unwrap()
                    .comfortable_hand_distance,
                0.48
            );
            assert!(!reset_calibration(&connection).unwrap().calibrated);
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_ranges_are_rejected() {
        let mut invalid = input();
        invalid.pinch_release_threshold = invalid.pinch_start_threshold;
        assert!(validate(&invalid).is_err());
        invalid = input();
        invalid.max_scale = invalid.min_scale;
        assert!(validate(&invalid).is_err());
    }

    fn register_input(identity: &str) -> RegisterEngineeringModelInput {
        RegisterEngineeringModelInput {
            model_identity: identity.into(),
            file_name: "motor.glb".into(),
            format: "GLB".into(),
            byte_size: 2048,
            components: vec![
                EngineeringModelComponentInput {
                    component_identity: "root/rotor:0".into(),
                    original_name: "Rotor".into(),
                    structural_path: "Motor/Rotor[0]".into(),
                    component_type: "Mesh".into(),
                    selectable: true,
                },
                EngineeringModelComponentInput {
                    component_identity: "root/shaft:0".into(),
                    original_name: "Shaft".into(),
                    structural_path: "Motor/Shaft[0]".into(),
                    component_type: "Mesh".into(),
                    selectable: true,
                },
            ],
        }
    }

    #[test]
    fn assembly_semantics_persist_and_stay_isolated_by_model() {
        let path = std::env::temp_dir().join(format!("azriel-assembly-{}.db", std::process::id()));
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            register_model(&mut connection, &register_input("model-a")).unwrap();
            register_model(&mut connection, &register_input("model-b")).unwrap();
            save_subsystem(
                &connection,
                &EngineeringSubsystemInput {
                    id: "drive-a".into(),
                    model_identity: "model-a".into(),
                    name: "Drive".into(),
                    description: "Power train".into(),
                    parent_subsystem_id: None,
                },
            )
            .unwrap();
            save_semantic(
                &connection,
                &ComponentSemanticInput {
                    model_identity: "model-a".into(),
                    component_identity: "root/rotor:0".into(),
                    semantic_label: "Rotor principal".into(),
                    subsystem_id: Some("drive-a".into()),
                    role: "Transmitir torque".into(),
                    description: "".into(),
                    notes: "".into(),
                },
            )
            .unwrap();
            save_relationship(
                &connection,
                &ComponentRelationshipInput {
                    id: "rel-a".into(),
                    model_identity: "model-a".into(),
                    source_component_identity: "root/rotor:0".into(),
                    target_component_identity: "root/shaft:0".into(),
                    relationship_type: "drives".into(),
                    description: "".into(),
                },
            )
            .unwrap();
            assert!(get_snapshot(&connection, "model-b")
                .unwrap()
                .semantics
                .is_empty());
        }
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            let snapshot = get_snapshot(&connection, "model-a").unwrap();
            assert_eq!(snapshot.semantics[0].semantic_label, "Rotor principal");
            assert_eq!(snapshot.subsystems.len(), 1);
            assert_eq!(snapshot.relationships[0].relationship_type, "drives");
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn assembly_rejects_self_duplicate_and_cross_model_relationships() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        register_model(&mut connection, &register_input("model-a")).unwrap();
        register_model(&mut connection, &register_input("model-b")).unwrap();
        let valid = ComponentRelationshipInput {
            id: "rel-1".into(),
            model_identity: "model-a".into(),
            source_component_identity: "root/rotor:0".into(),
            target_component_identity: "root/shaft:0".into(),
            relationship_type: "connected_to".into(),
            description: "".into(),
        };
        save_relationship(&connection, &valid).unwrap();
        let duplicate = ComponentRelationshipInput {
            id: "rel-2".into(),
            ..valid
        };
        assert!(save_relationship(&connection, &duplicate).is_err());
        let self_relation = ComponentRelationshipInput {
            id: "rel-3".into(),
            model_identity: "model-a".into(),
            source_component_identity: "root/rotor:0".into(),
            target_component_identity: "root/rotor:0".into(),
            relationship_type: "supports".into(),
            description: "".into(),
        };
        assert!(save_relationship(&connection, &self_relation).is_err());
        let cross_model = ComponentRelationshipInput {
            id: "rel-4".into(),
            model_identity: "model-b".into(),
            source_component_identity: "root/rotor:0".into(),
            target_component_identity: "missing-from-b".into(),
            relationship_type: "supports".into(),
            description: "".into(),
        };
        assert!(save_relationship(&connection, &cross_model).is_err());
        save_subsystem(
            &connection,
            &EngineeringSubsystemInput {
                id: "a".into(),
                model_identity: "model-a".into(),
                name: "A".into(),
                description: "".into(),
                parent_subsystem_id: None,
            },
        )
        .unwrap();
        save_subsystem(
            &connection,
            &EngineeringSubsystemInput {
                id: "b".into(),
                model_identity: "model-a".into(),
                name: "B".into(),
                description: "".into(),
                parent_subsystem_id: Some("a".into()),
            },
        )
        .unwrap();
        let cycle = EngineeringSubsystemInput {
            id: "a".into(),
            model_identity: "model-a".into(),
            name: "A".into(),
            description: "".into(),
            parent_subsystem_id: Some("b".into()),
        };
        assert!(save_subsystem(&connection, &cycle).is_err());
        let cross_semantic = ComponentSemanticInput {
            model_identity: "model-b".into(),
            component_identity: "root/rotor:0".into(),
            semantic_label: "Rotor".into(),
            subsystem_id: Some("a".into()),
            role: "".into(),
            description: "".into(),
            notes: "".into(),
        };
        assert!(save_semantic(&connection, &cross_semantic).is_err());
    }
}
