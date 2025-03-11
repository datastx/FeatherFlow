SELECT 
    c.id AS customer_id,
    c.name AS customer_name,
    c.email AS customer_email,
    COUNT(o.order_id) AS order_count,
    SUM(o.amount) AS total_amount
FROM staging.stg_customers c
LEFT JOIN staging.stg_orders o ON c.id = o.customer_id
GROUP BY c.id, c.name, c.email