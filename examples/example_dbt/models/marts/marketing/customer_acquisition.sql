{{ config(materialized='table') }}

-- Customer acquisition channel analysis

with customers as (
    select * from {{ ref('customers') }}
),

acquisition_analysis as (
    select
        signup_source as acquisition_channel,
        count(*) as total_customers,
        count(case when total_orders > 0 then 1 end) as converted_customers,
        sum(lifetime_revenue) as total_revenue,
        round(avg(lifetime_revenue), 2) as avg_customer_value,
        round(avg(total_orders), 2) as avg_orders_per_customer,
        count(case when customer_segment = 'VIP' then 1 end) as vip_customers,
        count(case when customer_segment = 'Regular' then 1 end) as regular_customers
    from customers
    group by signup_source
),

totals as (
    select sum(total_customers) as grand_total_customers
    from acquisition_analysis
)

select
    aa.acquisition_channel,
    aa.total_customers,
    round(aa.total_customers::numeric / nullif(t.grand_total_customers, 0) * 100, 2) as pct_of_total_customers,
    aa.converted_customers,
    round(aa.converted_customers::numeric / nullif(aa.total_customers, 0) * 100, 2) as conversion_rate_pct,
    aa.total_revenue,
    aa.avg_customer_value,
    aa.avg_orders_per_customer,
    aa.vip_customers,
    aa.regular_customers,

    -- Channel quality score
    case
        when aa.avg_customer_value >= 300 and aa.conversion_rate_pct >= 60 then 'Excellent'
        when aa.avg_customer_value >= 150 or aa.conversion_rate_pct >= 50 then 'Good'
        when aa.avg_customer_value >= 50 then 'Average'
        else 'Needs Improvement'
    end as channel_quality

from acquisition_analysis aa
cross join totals t
order by aa.total_revenue desc
