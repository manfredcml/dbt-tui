-- Enriches order items with product details

with order_items as (
    select * from {{ ref('stg_order_items') }}
),

products as (
    select * from {{ ref('stg_products') }}
),

enriched as (
    select
        oi.order_item_id,
        oi.order_id,
        oi.product_id,
        p.product_name,
        p.category,
        p.subcategory,
        oi.quantity,
        oi.unit_price,
        p.cost as unit_cost,
        oi.line_total,
        oi.quantity * p.cost as line_cost,
        oi.line_total - (oi.quantity * p.cost) as line_profit,
        oi.created_at
    from order_items oi
    left join products p on oi.product_id = p.product_id
)

select * from enriched
