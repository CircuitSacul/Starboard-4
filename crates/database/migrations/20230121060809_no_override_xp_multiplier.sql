UPDATE overrides SET overrides = (overrides::jsonb - 'xp_multiplier')::json;
