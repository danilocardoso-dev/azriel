-- Eleva apenas o antigo timeout padrao. Valores personalizados permanecem intactos.
UPDATE ai_settings
SET timeout_seconds = 90,
    updated_at = CURRENT_TIMESTAMP
WHERE id = 1
  AND timeout_seconds = 45;
