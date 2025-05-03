SELECT
    c.customer_id,
    c.name,
    c.email,
    c.credit_score,
    c.income_bracket,
    -- Account summary
    COUNT(DISTINCT a.account_id) AS account_count,
    SUM(a.current_balance) AS total_balance,
    -- Transaction summary
    COUNT(DISTINCT t.transaction_id) AS transaction_count,
    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,
    SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) AS total_income,
    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) / 
        NULLIF(COUNT(DISTINCT DATE_TRUNC('month', t.transaction_datetime)), 0) AS avg_monthly_spending,
    SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) / 
        NULLIF(COUNT(DISTINCT DATE_TRUNC('month', t.transaction_datetime)), 0) AS avg_monthly_income,
    -- Transaction patterns
    COUNT(DISTINCT CASE WHEN t.is_recurring THEN t.transaction_id ELSE NULL END) AS recurring_transaction_count,
    COUNT(DISTINCT t.category) AS category_count
FROM staging.stg_customers c
LEFT JOIN staging.stg_accounts a ON c.customer_id = a.customer_id
LEFT JOIN staging.stg_transactions t ON a.account_id = t.account_id
GROUP BY c.customer_id, c.name, c.email, c.credit_score, c.income_bracket