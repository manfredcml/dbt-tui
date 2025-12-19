{{ config(materialized='view') }}

with source as (
    select * from {{ source('raw', 'campaign_interactions') }}
),

renamed as (
    select
        id as interaction_id,
        customer_id,
        campaign_id,
        lower(interaction_type) as interaction_type,
        interaction_date,
        created_at
    from source
)

select * from renamed
