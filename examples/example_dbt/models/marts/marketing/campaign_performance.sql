{{ config(materialized='table') }}

-- Marketing campaign performance metrics

with campaigns as (
    select * from {{ ref('int_campaign_performance') }}
)

select
    campaign_id,
    campaign_name,
    channel,
    start_date,
    end_date,
    duration_days,
    budget,
    target_audience,
    total_interactions,
    impressions,
    clicks,
    email_opens,
    conversions,
    unique_customers,
    click_through_rate,
    conversion_rate,
    cost_per_conversion,

    -- Channel benchmarks
    case
        when channel in ('google_ads', 'facebook_ads') and click_through_rate >= 2 then 'Above Average'
        when channel in ('google_ads', 'facebook_ads') and click_through_rate >= 1 then 'Average'
        when channel in ('google_ads', 'facebook_ads') then 'Below Average'
        when channel = 'email' and click_through_rate >= 15 then 'Above Average'
        when channel = 'email' and click_through_rate >= 8 then 'Average'
        when channel = 'email' then 'Below Average'
        else 'N/A'
    end as performance_benchmark,

    -- ROI estimate (assuming $50 avg order value per conversion)
    case
        when conversions > 0
        then round((conversions * 50 - budget) / budget * 100, 2)
        else -100
    end as estimated_roi_pct

from campaigns
order by conversions desc, impressions desc
