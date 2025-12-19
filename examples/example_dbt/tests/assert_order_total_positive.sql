-- Test to ensure order totals are always positive (excluding cancelled orders)
-- Negative order totals would indicate pricing or discount calculation errors

select
    order_id,
    order_total
from {{ ref('orders') }}
where order_status != 'cancelled'
  and order_total <= 0
