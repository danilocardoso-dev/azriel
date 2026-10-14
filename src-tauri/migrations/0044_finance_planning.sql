CREATE TABLE finance_budgets (
  id TEXT PRIMARY KEY,
  year_month TEXT NOT NULL CHECK (length(year_month) = 7),
  scope TEXT NOT NULL CHECK (scope IN ('PERSONAL', 'PROFESSIONAL')),
  category_id TEXT NOT NULL REFERENCES financial_categories(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  limit_cents INTEGER NOT NULL CHECK (limit_cents > 0),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (year_month, scope, category_id)
);

CREATE TABLE finance_goals (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  scope TEXT NOT NULL CHECK (scope IN ('PERSONAL', 'PROFESSIONAL')),
  target_cents INTEGER NOT NULL CHECK (target_cents > 0),
  target_date TEXT,
  status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'COMPLETED', 'ARCHIVED')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE finance_goal_contributions (
  id TEXT PRIMARY KEY,
  goal_id TEXT NOT NULL REFERENCES finance_goals(id) ON UPDATE CASCADE ON DELETE CASCADE,
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  contribution_date TEXT NOT NULL,
  note TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE finance_goal_contribution_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  contribution_id TEXT NOT NULL,
  goal_id TEXT NOT NULL,
  previous_amount_cents INTEGER NOT NULL,
  previous_date TEXT NOT NULL,
  previous_note TEXT,
  corrected_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_finance_budgets_month ON finance_budgets(year_month, scope);
CREATE INDEX idx_finance_goals_status ON finance_goals(status, target_date);
CREATE INDEX idx_finance_goal_contributions_goal ON finance_goal_contributions(goal_id, contribution_date);

