{{ config(materialized='table') }}

-- Customer dimension with complete order history and metrics

with customers as (
    select * from {{ ref('stg_customers') }}
),

customer_orders as (
    select * from {{ ref('int_customer_orders') }}
),

countries as (
    select * from {{ ref('countries') }}
),

reviews as (
    select
        customer_id,
        count(*) as total_reviews,
        round(avg(rating), 2) as avg_rating_given
    from {{ ref('stg_reviews') }}
    group by customer_id
),

final as (
    select
        c.customer_id,
        c.first_name,
        c.last_name,
        c.full_name,
        c.email,
        c.phone,
        c.country_code,
        co_ref.country_name,
        co_ref.region,
        c.signup_source,

        -- Order metrics
        coalesce(cord.total_orders, 0) as total_orders,
        coalesce(cord.delivered_orders, 0) as delivered_orders,
        coalesce(cord.cancelled_orders, 0) as cancelled_orders,
        coalesce(cord.returned_orders, 0) as returned_orders,
        coalesce(cord.lifetime_items, 0) as lifetime_items,
        coalesce(cord.lifetime_revenue, 0) as lifetime_revenue,
        coalesce(cord.lifetime_profit, 0) as lifetime_profit,
        coalesce(cord.avg_order_value, 0) as avg_order_value,
        cord.first_order_date,
        cord.most_recent_order_date,
        coalesce(cord.customer_tenure_days, 0) as customer_tenure_days,

        -- Review metrics
        coalesce(r.total_reviews, 0) as total_reviews,
        r.avg_rating_given,

        -- Customer segment
        case
            when coalesce(cord.lifetime_revenue, 0) >= 500 then 'VIP'
            when coalesce(cord.lifetime_revenue, 0) >= 200 then 'Regular'
            when coalesce(cord.lifetime_revenue, 0) > 0 then 'New'
            else 'Prospect'
        end as customer_segment,

        c.created_at,
        c.updated_at
    from customers c
    left join customer_orders cord on c.customer_id = cord.customer_id
    left join countries co_ref on c.country_code = co_ref.country_code
    left join reviews r on c.customer_id = r.customer_id
)

select * from final
