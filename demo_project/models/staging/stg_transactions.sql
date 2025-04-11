SELECT
    transaction_id,
    account_id,
    merchant_id,
    card_id,
    transaction_datetime,
    amount,
    transaction_type,
    description,
    category,
    status,
    is_recurring,
    -- Time components for analysis
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