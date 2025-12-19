-- Test to ensure profit margins are within reasonable bounds (-50% to 200%)
-- Extreme margins might indicate pricing or cost data errors

select
    product_id,
    product_name,
    margin_pct
from {{ ref('stg_products') }}
where margin_pct < -50 or margin_pct > 200
