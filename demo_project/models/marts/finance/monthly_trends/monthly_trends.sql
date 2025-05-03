WITH monthly_data AS (
    SELECT
        t.year,
        t.month,
        DATE_TRUNC('month', t.transaction_datetime) AS month_start,
        COUNT(t.transaction_id) AS transaction_count,
        COUNT(DISTINCT t.account_id) AS active_accounts,
        SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,
        SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) AS total_income,
        AVG(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE NULL END) AS avg_spend_amount,
        COUNT(DISTINCT m.category) AS category_count
    FROM staging.stg_transactions t
    JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
    GROUP BY t.year, t.month, DATE_TRUNC('month', t.transaction_datetime)
)

SELECT
    d.year,
    d.month,
    d.month_start,
    d.transaction_count,
    d.active_accounts,
    d.total_spending,
    d.total_income,
    d.avg_spend_amount,
    d.category_count,
    -- Month over month growth
    d.total_spending - LAG(d.total_spending) OVER (ORDER BY d.year, d.month) AS mom_spending_change,
    (d.total_spending - LAG(d.total_spending) OVER (ORDER BY d.year, d.month)) / 
        NULLIF(LAG(d.total_spending) OVER (ORDER BY d.year, d.month), 0) * 100 AS mom_spending_pct_change,
    -- Year over year comparison (for same month)
    d.total_spending - LAG(d.total_spending) OVER (
        PARTITION BY d.month ORDER BY d.year
    ) AS yoy_spending_change,
    (d.total_spending - LAG(d.total_spending) OVER (
        PARTITION BY d.month ORDER BY d.year
    )) / NULLIF(LAG(d.total_spending) OVER (
        PARTITION BY d.month ORDER BY d.year
    ), 0) * 100 AS yoy_spending_pct_change,
    -- 3-month rolling average
    AVG(d.total_spending) OVER (
        ORDER BY d.year, d.month
        ROWS BETWEEN 2 PRECEDING AND CURRENT ROW
    ) AS spending_3month_rolling_avg
FROM monthly_data d
ORDER BY d.year, d.month