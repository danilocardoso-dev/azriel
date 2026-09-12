ALTER TABLE market_position_events
ADD COLUMN execution_id INTEGER REFERENCES market_executions(id) ON DELETE RESTRICT;
