SELECT
    DATE(t.transaction_datetime) AS date,
    t.day_of_week,
    CASE 
        WHEN t.day_of_week IN (0, 6) THEN 'Weekend'
        ELSE 'Weekday'
    END AS day_type,
    COUNT(t.transaction_id) AS transaction_count,
    COUNT(DISTINCT t.account_id) AS active_accounts,
    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,
    SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) AS total_income,
    AVG(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE NULL END) AS avg_spend_amount,
    -- Day over day metrics
    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) - 
        LAG(SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END)) OVER 
        (ORDER BY DATE(t.transaction_datetime)) AS spending_change_from_previous_day,
    -- Number of merchant categories
    COUNT(DISTINCT m.category) AS active_merchant_categories,
    -- Moving averages
    AVG(SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END)) OVER (
        ORDER BY DATE(t.transaction_datetime)
        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
    ) AS spending_7day_moving_avg
FROM staging.stg_transactions t
JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
GROUP BY DATE(t.transaction_datetime), t.day_of_week
ORDER BY date