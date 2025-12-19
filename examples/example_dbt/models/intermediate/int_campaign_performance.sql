-- Calculates campaign performance metrics

with campaigns as (
    select * from {{ ref('stg_campaigns') }}
),

interactions as (
    select * from {{ ref('stg_campaign_interactions') }}
),

interaction_metrics as (
    select
        campaign_id,
        count(*) as total_interactions,
        count(case when interaction_type = 'impression' then 1 end) as impressions,
        count(case when interaction_type = 'click' then 1 end) as clicks,
        count(case when interaction_type = 'email_open' then 1 end) as email_opens,
        count(case when interaction_type = 'conversion' then 1 end) as conversions,
        count(distinct customer_id) as unique_customers
    from interactions
    group by campaign_id
),

final as (
    select
        c.campaign_id,
        c.campaign_name,
        c.channel,
        c.start_date,
        c.end_date,
        c.duration_days,
        c.budget,
        c.target_audience,
        coalesce(im.total_interactions, 0) as total_interactions,
        coalesce(im.impressions, 0) as impressions,
        coalesce(im.clicks, 0) as clicks,
        coalesce(im.email_opens, 0) as email_opens,
        coalesce(im.conversions, 0) as conversions,
        coalesce(im.unique_customers, 0) as unique_customers,
        case
            when coalesce(im.impressions, 0) > 0
            then round(coalesce(im.clicks, 0)::numeric / im.impressions * 100, 2)
            else 0
        end as click_through_rate,
        case
            when coalesce(im.clicks, 0) > 0
            then round(coalesce(im.conversions, 0)::numeric / im.clicks * 100, 2)
            else 0
        end as conversion_rate,
        case
            when coalesce(im.conversions, 0) > 0
            then round(c.budget / im.conversions, 2)
            else null
        end as cost_per_conversion
    from campaigns c
    left join interaction_metrics im on c.campaign_id = im.campaign_id
)

select * from final
