{{ config(materialized='table') }}

-- Product dimension with sales and review metrics

with products as (
    select * from {{ ref('stg_products') }}
),

order_items as (
    select * from {{ ref('int_order_items_enriched') }}
),

reviews as (
    select
        product_id,
        count(*) as review_count,
        round(avg(rating), 2) as avg_rating,
        count(case when sentiment = 'positive' then 1 end) as positive_reviews,
        count(case when sentiment = 'negative' then 1 end) as negative_reviews
    from {{ ref('stg_reviews') }}
    group by product_id
),

sales_metrics as (
    select
        product_id,
        count(distinct order_id) as orders_containing_product,
        sum(quantity) as total_units_sold,
        sum(line_total) as total_revenue,
        sum(line_profit) as total_profit,
        avg(unit_price) as avg_selling_price
    from order_items
    group by product_id
),

final as (
    select
        p.product_id,
        p.product_name,
        p.category,
        p.subcategory,
        p.price,
        p.cost,
        p.gross_margin,
        p.margin_pct,
        p.sku,
        p.is_active,

        -- Sales metrics
        coalesce(sm.orders_containing_product, 0) as orders_containing_product,
        coalesce(sm.total_units_sold, 0) as total_units_sold,
        coalesce(sm.total_revenue, 0) as total_revenue,
        coalesce(sm.total_profit, 0) as total_profit,
        sm.avg_selling_price,

        -- Review metrics
        coalesce(r.review_count, 0) as review_count,
        r.avg_rating,
        coalesce(r.positive_reviews, 0) as positive_reviews,
        coalesce(r.negative_reviews, 0) as negative_reviews,

        -- Product ranking
        case
            when coalesce(sm.total_revenue, 0) >= 500 then 'Top Seller'
            when coalesce(sm.total_revenue, 0) >= 100 then 'Good Performer'
            when coalesce(sm.total_revenue, 0) > 0 then 'Regular'
            else 'No Sales'
        end as sales_tier,

        p.created_at
    from products p
    left join sales_metrics sm on p.product_id = sm.product_id
    left join reviews r on p.product_id = r.product_id
)

select * from final
