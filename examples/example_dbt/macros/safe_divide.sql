{# Safely divide two numbers, returning null instead of error on division by zero #}
{% macro safe_divide(numerator, denominator, precision=2) %}
    case
        when {{ denominator }} = 0 or {{ denominator }} is null then null
        else round(({{ numerator }})::numeric / ({{ denominator }})::numeric, {{ precision }})
    end
{% endmacro %}
