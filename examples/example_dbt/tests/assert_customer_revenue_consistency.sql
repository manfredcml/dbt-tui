-- Test to ensure VIP customers actually have the required revenue
-- VIP segment requires >= $500 lifetime revenue

select
    customer_id,
    customer_segment,
    lifetime_revenue
from {{ ref('customers') }}
where customer_segment = 'VIP'
  and lifetime_revenue < 500
