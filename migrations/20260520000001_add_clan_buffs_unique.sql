ALTER TABLE clan_buffs ADD CONSTRAINT IF NOT EXISTS clan_buffs_clan_id_buff_name_key UNIQUE (clan_id, buff_name);
