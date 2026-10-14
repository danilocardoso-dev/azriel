CREATE TABLE financial_categories (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL COLLATE NOCASE,
  scope TEXT NOT NULL CHECK (scope IN ('PERSONAL', 'PROFESSIONAL')),
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (name, scope)
);

CREATE TABLE financial_transactions (
  id TEXT PRIMARY KEY,
  transaction_type TEXT NOT NULL CHECK (transaction_type IN ('INCOME', 'EXPENSE')),
  scope TEXT NOT NULL CHECK (scope IN ('PERSONAL', 'PROFESSIONAL')),
  description TEXT NOT NULL,
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  category_id TEXT NOT NULL REFERENCES financial_categories(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  competence_date TEXT NOT NULL,
  due_date TEXT,
  paid_at TEXT,
  status TEXT NOT NULL CHECK (status IN ('PLANNED', 'PAID', 'CANCELLED')),
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (status <> 'PAID' OR paid_at IS NOT NULL)
);

CREATE INDEX idx_financial_transactions_competence ON financial_transactions(competence_date);
CREATE INDEX idx_financial_transactions_paid ON financial_transactions(paid_at);
CREATE INDEX idx_financial_transactions_due ON financial_transactions(due_date);
CREATE INDEX idx_financial_transactions_filters ON financial_transactions(status, scope, transaction_type, category_id);

INSERT INTO financial_categories(id, name, scope) VALUES
  ('personal-housing', 'Moradia', 'PERSONAL'),
  ('personal-food', 'Alimentação', 'PERSONAL'),
  ('personal-transport', 'Transporte', 'PERSONAL'),
  ('personal-health', 'Saúde', 'PERSONAL'),
  ('personal-education', 'Educação', 'PERSONAL'),
  ('personal-leisure', 'Lazer', 'PERSONAL'),
  ('personal-subscriptions', 'Assinaturas pessoais', 'PERSONAL'),
  ('personal-other', 'Outros', 'PERSONAL'),
  ('professional-tools', 'Ferramentas', 'PROFESSIONAL'),
  ('professional-servers', 'Servidores', 'PROFESSIONAL'),
  ('professional-hosting', 'Hospedagem', 'PROFESSIONAL'),
  ('professional-domains', 'Domínios', 'PROFESSIONAL'),
  ('professional-software', 'Software', 'PROFESSIONAL'),
  ('professional-apis', 'APIs', 'PROFESSIONAL'),
  ('professional-digital-services', 'Serviços digitais', 'PROFESSIONAL'),
  ('professional-equipment', 'Equipamentos', 'PROFESSIONAL'),
  ('professional-mei-taxes', 'Tributos MEI', 'PROFESSIONAL'),
  ('professional-other', 'Outros custos operacionais', 'PROFESSIONAL');
