WITH category_spending AS (
    SELECT
        t.year,
        t.month,
        DATE_TRUNC('month', t.transaction_datetime) AS month_start,
        m.category,
        COUNT(t.transaction_id) AS transaction_count,
        SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS category_spending
    FROM staging.stg_transactions t
    JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
    WHERE t.amount < 0  -- Only outgoing transactions
    GROUP BY t.year, t.month, DATE_TRUNC('month', t.transaction_datetime), m.category
),

monthly_totals AS (
    SELECT
        year,
        month,
        month_start,
        SUM(category_spending) AS total_monthly_spending
    FROM category_spending
    GROUP BY year, month, month_start
)

SELECT
    cs.year,
    cs.month,
    cs.month_start,
    cs.category,
    cs.transaction_count,
    cs.category_spending,
    -- Category percentage of monthly spending
    cs.category_spending / NULLIF(mt.total_monthly_spending, 0) * 100 AS category_pct_of_total,
    -- Rank categories within each month
    ROW_NUMBER() OVER (PARTITION BY cs.year, cs.month ORDER BY cs.category_spending DESC) AS category_rank,
    -- Previous year's spending in same category and month
    LAG(cs.category_spending) OVER (
        PARTITION BY cs.month, cs.category ORDER BY cs.year
    ) AS prev_year_category_spending,
    -- Year over year category growth
    (cs.category_spending - LAG(cs.category_spending) OVER (
        PARTITION BY cs.month, cs.category ORDER BY cs.year
    )) / NULLIF(LAG(cs.category_spending) OVER (
        PARTITION BY cs.month, cs.category ORDER BY cs.year
    ), 0) * 100 AS yoy_category_pct_change,
    -- Seasonal analysis, we should add a data test to make sure that this is never null
    CASE 
        WHEN cs.month IN (12, 1, 2) THEN 'Winter'
        WHEN cs.month IN (3, 4, 5) THEN 'Spring'
        WHEN cs.month IN (6, 7, 8) THEN 'Summer'
        WHEN cs.month IN (9, 10, 11) THEN 'Fall' 
        ELSE null
    END AS season
FROM category_spending cs
JOIN monthly_totals mt ON cs.year = mt.year AND cs.month = mt.month
ORDER BY cs.year, cs.month, category_spending DESC