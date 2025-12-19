-- Test to ensure total_orders is always >= 0 in customers table
-- A negative order count would indicate a data integrity issue

select
    customer_id,
    total_orders
from {{ ref('customers') }}
where total_orders < 0
