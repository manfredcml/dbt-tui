{{ config(materialized='view') }}

-- This is a change

with source as (
    select * from {{ source('raw', 'campaigns') }}
),

renamed as (
    select
        id as campaign_id,
        name as campaign_name,
        lower(channel) as channel,
        start_date,
        end_date,
        end_date - start_date as duration_days,
        budget,
        target_audience,
        created_at
    from source
)

select * from renamed
