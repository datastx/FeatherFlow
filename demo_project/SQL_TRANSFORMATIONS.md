# FeatherFlow Financial Demo SQL Transformations

This document contains the SQL transformations that will be used in the FeatherFlow Financial Demo. These transformations demonstrate how to leverage DuckDB and FeatherFlow to analyze financial data, with a special focus on time-series analysis.

## Staging Models

These models prepare the raw data for further transformation by cleaning and standardizing the datasets.

### `stg_customers.sql`

```sql
SELECT
    customer_id,
    name,
    email,
    address,
    CAST(registration_date AS DATE) AS registration_date,
    credit_score,
    income_bracket
FROM raw_data.customers
```

### `stg_accounts.sql`

```sql
SELECT
    account_id,
    customer_id,
    account_type,
    CAST(open_date AS DATE) AS open_date,
    status,
    currency,
    initial_balance,
    current_balance
FROM raw_data.accounts
```

### `stg_transactions.sql`

```sql
SELECT
    transaction_id,
    account_id,
    merchant_id,
    card_id,
    CAST(transaction_datetime AS TIMESTAMP) AS transaction_datetime,
    amount,
    transaction_type,
    description,
    category,
    status,
    is_recurring,
    -- Time-series components
    day_of_week,
    month,
    year,
    time_of_day,
    -- Additional time components
    EXTRACT(DAY FROM transaction_datetime) AS day_of_month,
    CASE 
        WHEN EXTRACT(MONTH FROM transaction_datetime) IN (12, 1, 2) THEN 'Winter'
        WHEN EXTRACT(MONTH FROM transaction_datetime) IN (3, 4, 5) THEN 'Spring'
        WHEN EXTRACT(MONTH FROM transaction_datetime) IN (6, 7, 8) THEN 'Summer'
        ELSE 'Fall'
    END AS season
FROM raw_data.transactions
```

### `stg_merchants.sql`

```sql
SELECT
    merchant_id,
    name,
    category,
    location,
    is_online,
    popularity_score
FROM raw_data.merchants
```

### `stg_credit_cards.sql`

```sql
SELECT
    card_id,
    customer_id,
    card_type,
    credit_limit,
    CAST(issue_date AS DATE) AS issue_date,
    CAST(expiry_date AS DATE) AS expiry_date,
    status
FROM raw_data.credit_cards
```

### `stg_loans.sql`

```sql
SELECT
    loan_id,
    customer_id,
    loan_type,
    principal_amount,
    interest_rate,
    term_months,
    CAST(start_date AS DATE) AS start_date,
    status
FROM raw_data.loans
```

## Mart Models - Core

These models provide core business metrics and aggregates.

### `customer_summary.sql`

```sql
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
    -- Credit info
    COUNT(DISTINCT cc.card_id) AS credit_card_count,
    SUM(cc.credit_limit) AS total_credit_limit,
    -- Loan info
    COUNT(DISTINCT l.loan_id) AS loan_count,
    SUM(l.principal_amount) AS total_loan_amount
FROM staging.stg_customers c
LEFT JOIN staging.stg_accounts a ON c.customer_id = a.customer_id
LEFT JOIN staging.stg_transactions t ON a.account_id = t.account_id
LEFT JOIN staging.stg_credit_cards cc ON c.customer_id = cc.customer_id
LEFT JOIN staging.stg_loans l ON c.customer_id = l.customer_id
GROUP BY c.customer_id, c.name, c.email, c.credit_score, c.income_bracket
```

### `merchant_summary.sql`

```sql
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
```

## Mart Models - Finance

These models focus on financial analysis, especially time-series aspects.

### `daily_trends.sql`

```sql
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
```

### `monthly_trends.sql`

```sql
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
```

### `spending_categories.sql`

```sql
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
    -- Seasonal analysis
    CASE 
        WHEN cs.month IN (12, 1, 2) THEN 'Winter'
        WHEN cs.month IN (3, 4, 5) THEN 'Spring'
        WHEN cs.month IN (6, 7, 8) THEN 'Summer'
        ELSE 'Fall'
    END AS season
FROM category_spending cs
JOIN monthly_totals mt ON cs.year = mt.year AND cs.month = mt.month
ORDER BY cs.year, cs.month, category_spending DESC
```

### `recurring_analysis.sql`

```sql
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
```

### `seasonal_patterns.sql`

```sql
WITH seasonal_spending AS (
    SELECT
        CASE 
            WHEN t.month IN (12, 1, 2) THEN 'Winter'
            WHEN t.month IN (3, 4, 5) THEN 'Spring'
            WHEN t.month IN (6, 7, 8) THEN 'Summer'
            ELSE 'Fall'
        END AS season,
        t.year,
        m.category,
        SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,
        COUNT(t.transaction_id) AS transaction_count
    FROM staging.stg_transactions t
    JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
    WHERE t.amount < 0  -- Only consider spending
    GROUP BY 
        CASE 
            WHEN t.month IN (12, 1, 2) THEN 'Winter'
            WHEN t.month IN (3, 4, 5) THEN 'Spring'
            WHEN t.month IN (6, 7, 8) THEN 'Summer'
            ELSE 'Fall'
        END,
        t.year,
        m.category
)

SELECT
    ss.season,
    ss.year,
    ss.category,
    ss.total_spending,
    ss.transaction_count,
    -- Rank categories within each season
    ROW_NUMBER() OVER (
        PARTITION BY ss.season, ss.year 
        ORDER BY ss.total_spending DESC
    ) AS category_rank_in_season,
    -- Season over season comparison
    LAG(ss.total_spending) OVER (
        PARTITION BY ss.category
        ORDER BY ss.year, 
            CASE 
                WHEN ss.season = 'Winter' THEN 1
                WHEN ss.season = 'Spring' THEN 2
                WHEN ss.season = 'Summer' THEN 3
                ELSE 4
            END
    ) AS prev_season_spending,
    -- Growth rate from previous same season
    (ss.total_spending - LAG(ss.total_spending) OVER (
        PARTITION BY ss.category, ss.season
        ORDER BY ss.year
    )) / NULLIF(LAG(ss.total_spending) OVER (
        PARTITION BY ss.category, ss.season
        ORDER BY ss.year
    ), 0) * 100 AS yoy_season_growth_pct
FROM seasonal_spending ss
ORDER BY 
    ss.year,
    CASE 
        WHEN ss.season = 'Winter' THEN 1
        WHEN ss.season = 'Spring' THEN 2
        WHEN ss.season = 'Summer' THEN 3
        ELSE 4
    END,
    ss.total_spending DESC
```

## Advanced Time-Series Models

These models demonstrate more sophisticated time-series analysis techniques.

### `spending_forecasting.sql`

```sql
WITH monthly_category_spending AS (
    SELECT
        DATE_TRUNC('month', t.transaction_datetime) AS month_date,
        t.year,
        t.month,
        m.category,
        SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS category_spending
    FROM staging.stg_transactions t
    JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
    WHERE t.amount < 0  -- Only spending
    GROUP BY DATE_TRUNC('month', t.transaction_datetime), t.year, t.month, m.category
),

trend_analysis AS (
    SELECT
        mcs.month_date,
        mcs.year,
        mcs.month,
        mcs.category,
        mcs.category_spending,
        -- Moving averages
        AVG(mcs.category_spending) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
            ROWS BETWEEN 2 PRECEDING AND CURRENT ROW
        ) AS ma_3month,
        AVG(mcs.category_spending) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
            ROWS BETWEEN 5 PRECEDING AND CURRENT ROW
        ) AS ma_6month,
        -- Trend calculation
        (mcs.category_spending - LAG(mcs.category_spending, 1) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
        )) AS month_trend,
        (mcs.category_spending - LAG(mcs.category_spending, 12) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
        )) AS year_trend,
        -- Simple extrapolation (linear)
        mcs.category_spending + 
            (mcs.category_spending - LAG(mcs.category_spending, 1) OVER (
                PARTITION BY mcs.category
                ORDER BY mcs.year, mcs.month
            )) AS next_month_forecast_simple,
        -- Weighted moving average forecast
        (0.5 * mcs.category_spending) +
        (0.3 * LAG(mcs.category_spending, 1) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
        )) +
        (0.2 * LAG(mcs.category_spending, 2) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
        )) AS next_month_forecast_wma,
        -- Seasonal forecast (using same month last year + trend)
        LAG(mcs.category_spending, 12) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
        ) + 
        ((LAG(mcs.category_spending, 1) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
        ) - LAG(mcs.category_spending, 13) OVER (
            PARTITION BY mcs.category
            ORDER BY mcs.year, mcs.month
        )) / 12) AS next_month_forecast_seasonal
    FROM monthly_category_spending mcs
)

SELECT
    ta.month_date,
    ta.year,
    ta.month,
    ta.category,
    ta.category_spending,
    ta.ma_3month,
    ta.ma_6month,
    ta.month_trend,
    ta.year_trend,
    ta.next_month_forecast_simple,
    ta.next_month_forecast_wma,
    ta.next_month_forecast_seasonal,
    -- Forecasting accuracy (if we have actual data to compare)
    LEAD(ta.category_spending, 1) OVER (
        PARTITION BY ta.category
        ORDER BY ta.year, ta.month
    ) AS next_month_actual,
    -- Error calculation
    ABS(ta.next_month_forecast_simple - LEAD(ta.category_spending, 1) OVER (
        PARTITION BY ta.category
        ORDER BY ta.year, ta.month
    )) / NULLIF(LEAD(ta.category_spending, 1) OVER (
        PARTITION BY ta.category
        ORDER BY ta.year, ta.month
    ), 0) AS simple_forecast_error_pct,
    ABS(ta.next_month_forecast_wma - LEAD(ta.category_spending, 1) OVER (
        PARTITION BY ta.category
        ORDER BY ta.year, ta.month
    )) / NULLIF(LEAD(ta.category_spending, 1) OVER (
        PARTITION BY ta.category
        ORDER BY ta.year, ta.month
    ), 0) AS wma_forecast_error_pct,
    ABS(ta.next_month_forecast_seasonal - LEAD(ta.category_spending, 1) OVER (
        PARTITION BY ta.category
        ORDER BY ta.year, ta.month
    )) / NULLIF(LEAD(ta.category_spending, 1) OVER (
        PARTITION BY ta.category
        ORDER BY ta.year, ta.month
    ), 0) AS seasonal_forecast_error_pct
FROM trend_analysis ta
ORDER BY ta.category, ta.year, ta.month
```

These SQL transformations demonstrate a range of time-series techniques for analyzing financial data in FeatherFlow:

1. **Basic temporal decomposition**: Breaking dates into components (day, month, year) for aggregation
2. **Window functions**: Using LAG, LEAD, and moving averages to analyze trends over time
3. **Seasonal analysis**: Comparing spending patterns across different seasons
4. **Year-over-year comparisons**: Analyzing growth rates compared to the same period last year
5. **Forecasting techniques**: Using moving averages and seasonal adjustments to predict future spending
6. **Recurring pattern detection**: Identifying and analyzing regular financial patterns

These transformations create a foundation for sophisticated financial analytics that could be extended with more advanced time-series techniques in future iterations.