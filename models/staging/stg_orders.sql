SELECT 
    id AS order_id,
    customer_id,
    order_date,
    status,
    amount
FROM raw_data.orders