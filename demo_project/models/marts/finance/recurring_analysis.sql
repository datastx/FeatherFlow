WITH recurring_transactions AS (
    SELECT
        t.account_id,
        t.merchant_id,
        m.name AS merchant_name,
        m.category AS merchant_category,
        COUNT(t.transaction_id) AS transaction_count,
        MIN(ABS(t.amount)) AS min_amount,
        MAX(ABS(t.amount)) AS max_amount,
        AVG(ABS(t.amount)) AS avg_amount,
        MIN(t.transaction_datetime) AS first_transaction,
        MAX(t.transaction_datetime) AS last_transaction,
        -- Calculate time between transactions
        (DATEDIFF('day', MIN(t.transaction_datetime), MAX(t.transaction_datetime))) / 
            NULLIF(COUNT(t.transaction_id) - 1, 0) AS avg_days_between
    FROM staging.stg_transactions t
    JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
    WHERE t.is_recurring = TRUE
    GROUP BY t.account_id, t.merchant_id, m.name, m.category
    HAVING COUNT(t.transaction_id) > 1  -- Must have more than one transaction
),

account_summary AS (
    SELECT
        rt.account_id,
        COUNT(DISTINCT rt.merchant_id) AS recurring_merchant_count,
        SUM(rt.avg_amount) AS total_recurring_monthly_expenses
    FROM recurring_transactions rt
    WHERE rt.avg_days_between BETWEEN 28 AND 31  -- Monthly recurring
    GROUP BY rt.account_id
)

SELECT
    c.customer_id,
    c.name AS customer_name,
    a.account_id,
    a.account_type,
    asm.recurring_merchant_count,
    asm.total_recurring_monthly_expenses,
    -- Income metrics for comparison
    (SELECT SUM(t.amount) / COUNT(DISTINCT (t.year || '-' || t.month))
     FROM staging.stg_transactions t
     WHERE t.account_id = a.account_id AND t.amount > 0) AS avg_monthly_income,
    -- Ratio of recurring expenses to income
    asm.total_recurring_monthly_expenses / NULLIF(
        (SELECT SUM(t.amount) / COUNT(DISTINCT (t.year || '-' || t.month))
         FROM staging.stg_transactions t
         WHERE t.account_id = a.account_id AND t.amount > 0), 0
    ) * 100 AS recurring_expense_to_income_pct,
    -- Categorize customers by recurring expense ratio
    CASE
        WHEN asm.total_recurring_monthly_expenses / NULLIF(
            (SELECT SUM(t.amount) / COUNT(DISTINCT (t.year || '-' || t.month))
             FROM staging.stg_transactions t
             WHERE t.account_id = a.account_id AND t.amount > 0), 0
        ) * 100 > 50 THEN 'High Fixed Expenses'
        WHEN asm.total_recurring_monthly_expenses / NULLIF(
            (SELECT SUM(t.amount) / COUNT(DISTINCT (t.year || '-' || t.month))
             FROM staging.stg_transactions t
             WHERE t.account_id = a.account_id AND t.amount > 0), 0
        ) * 100 BETWEEN 30 AND 50 THEN 'Moderate Fixed Expenses'
        ELSE 'Low Fixed Expenses'
    END AS expense_category
FROM staging.stg_customers c
JOIN staging.stg_accounts a ON c.customer_id = a.customer_id
JOIN account_summary asm ON a.account_id = asm.account_id
ORDER BY recurring_expense_to_income_pct DESC