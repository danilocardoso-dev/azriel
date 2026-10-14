CREATE TABLE finance_contracts (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  counterparty TEXT NOT NULL,
  description TEXT,
  scope TEXT NOT NULL CHECK (scope IN ('PERSONAL', 'PROFESSIONAL')),
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  currency TEXT NOT NULL DEFAULT 'BRL' CHECK (currency = 'BRL'),
  frequency TEXT NOT NULL CHECK (frequency IN ('MONTHLY', 'WEEKLY', 'YEARLY', 'ONE_TIME')),
  start_date TEXT NOT NULL,
  end_date TEXT,
  first_due_date TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'PAUSED', 'ENDED', 'CANCELLED')),
  category_id TEXT NOT NULL REFERENCES financial_categories(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  notes TEXT,
  resume_date TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (end_date IS NULL OR end_date >= start_date)
);

CREATE TABLE finance_recurrences (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  scope TEXT NOT NULL CHECK (scope IN ('PERSONAL', 'PROFESSIONAL')),
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  estimated INTEGER NOT NULL DEFAULT 0 CHECK (estimated IN (0, 1)),
  frequency TEXT NOT NULL CHECK (frequency IN ('MONTHLY', 'WEEKLY', 'YEARLY', 'ONE_TIME')),
  start_date TEXT NOT NULL,
  end_date TEXT,
  first_due_date TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'PAUSED', 'ENDED', 'CANCELLED')),
  category_id TEXT NOT NULL REFERENCES financial_categories(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  notes TEXT,
  resume_date TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (end_date IS NULL OR end_date >= start_date)
);

CREATE TABLE finance_loans (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  creditor TEXT NOT NULL,
  purpose TEXT,
  scope TEXT NOT NULL CHECK (scope IN ('PERSONAL', 'PROFESSIONAL')),
  original_principal_cents INTEGER NOT NULL CHECK (original_principal_cents > 0),
  informed_balance_cents INTEGER NOT NULL CHECK (informed_balance_cents >= 0),
  balance_as_of TEXT NOT NULL,
  installment_count INTEGER NOT NULL CHECK (installment_count > 0),
  interest_rate_basis_points INTEGER,
  interest_rate_period TEXT CHECK (interest_rate_period IS NULL OR interest_rate_period IN ('MONTHLY', 'YEARLY')),
  status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'PAUSED', 'PAID', 'CANCELLED')),
  category_id TEXT NOT NULL REFERENCES financial_categories(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  notes TEXT,
  resume_date TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK ((interest_rate_basis_points IS NULL AND interest_rate_period IS NULL) OR (interest_rate_basis_points IS NOT NULL AND interest_rate_basis_points >= 0 AND interest_rate_period IS NOT NULL))
);

CREATE TABLE finance_loan_installments (
  id TEXT PRIMARY KEY,
  loan_id TEXT NOT NULL REFERENCES finance_loans(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  installment_number INTEGER NOT NULL CHECK (installment_number > 0),
  due_date TEXT NOT NULL,
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  principal_cents INTEGER CHECK (principal_cents IS NULL OR principal_cents >= 0),
  interest_cents INTEGER CHECK (interest_cents IS NULL OR interest_cents >= 0),
  fees_cents INTEGER CHECK (fees_cents IS NULL OR fees_cents >= 0),
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (loan_id, installment_number)
);

CREATE TABLE finance_commitments (
  id TEXT PRIMARY KEY,
  source_type TEXT NOT NULL CHECK (source_type IN ('CONTRACT', 'RECURRING_EXPENSE', 'LOAN_INSTALLMENT')),
  contract_id TEXT REFERENCES finance_contracts(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  recurrence_id TEXT REFERENCES finance_recurrences(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  loan_installment_id TEXT REFERENCES finance_loan_installments(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  occurrence_key TEXT NOT NULL,
  direction TEXT NOT NULL CHECK (direction IN ('INCOME', 'EXPENSE')),
  scope TEXT NOT NULL CHECK (scope IN ('PERSONAL', 'PROFESSIONAL')),
  category_id TEXT NOT NULL REFERENCES financial_categories(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  expected_amount_cents INTEGER NOT NULL CHECK (expected_amount_cents > 0),
  estimated INTEGER NOT NULL DEFAULT 0 CHECK (estimated IN (0, 1)),
  competence_date TEXT NOT NULL,
  due_date TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'SETTLED', 'CANCELLED')),
  transaction_id TEXT UNIQUE REFERENCES financial_transactions(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  settled_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (
    (source_type = 'CONTRACT' AND contract_id IS NOT NULL AND recurrence_id IS NULL AND loan_installment_id IS NULL) OR
    (source_type = 'RECURRING_EXPENSE' AND contract_id IS NULL AND recurrence_id IS NOT NULL AND loan_installment_id IS NULL) OR
    (source_type = 'LOAN_INSTALLMENT' AND contract_id IS NULL AND recurrence_id IS NULL AND loan_installment_id IS NOT NULL)
  ),
  CHECK ((status = 'SETTLED' AND transaction_id IS NOT NULL AND settled_at IS NOT NULL) OR (status <> 'SETTLED' AND transaction_id IS NULL AND settled_at IS NULL))
);

CREATE UNIQUE INDEX idx_finance_commitment_contract_occurrence ON finance_commitments(contract_id, occurrence_key) WHERE contract_id IS NOT NULL;
CREATE UNIQUE INDEX idx_finance_commitment_recurrence_occurrence ON finance_commitments(recurrence_id, occurrence_key) WHERE recurrence_id IS NOT NULL;
CREATE UNIQUE INDEX idx_finance_commitment_installment_occurrence ON finance_commitments(loan_installment_id, occurrence_key) WHERE loan_installment_id IS NOT NULL;
CREATE INDEX idx_finance_contracts_status ON finance_contracts(status, start_date, end_date);
CREATE INDEX idx_finance_recurrences_status ON finance_recurrences(status, start_date, end_date);
CREATE INDEX idx_finance_loans_status ON finance_loans(status, balance_as_of);
CREATE INDEX idx_finance_installments_due ON finance_loan_installments(due_date, loan_id);
CREATE INDEX idx_finance_commitments_due ON finance_commitments(status, due_date);
CREATE INDEX idx_finance_commitments_origin ON finance_commitments(source_type, contract_id, recurrence_id, loan_installment_id);
CREATE INDEX idx_finance_commitments_scope ON finance_commitments(scope, direction, status);
