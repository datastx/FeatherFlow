SELECT
    m.merchant_id,
    m.name,
    m.category,
    m.location,
    m.is_online,
    -- Transaction metrics
    COUNT(t.transaction_id) AS transaction_count,
    COUNT(DISTINCT t.account_id) AS unique_customer_count,
    SUM(ABS(t.amount)) AS transaction_volume,
    AVG(ABS(t.amount)) AS avg_transaction_amount,
    -- Time patterns
    AVG(CASE WHEN t.day_of_week IN (0, 6) THEN ABS(t.amount) ELSE NULL END) AS avg_weekend_amount,
    AVG(CASE WHEN t.day_of_week BETWEEN 1 AND 5 THEN ABS(t.amount) ELSE NULL END) AS avg_weekday_amount,
    -- Recurring metrics
    SUM(CASE WHEN t.is_recurring THEN 1 ELSE 0 END) AS recurring_transaction_count,
    -- Monthly stats
    COUNT(DISTINCT (t.year || '-' || t.month)) AS active_months
FROM staging.stg_merchants m
LEFT JOIN staging.stg_transactions t ON m.merchant_id = t.merchant_id
GROUP BY m.merchant_id, m.name, m.category, m.location, m.is_online