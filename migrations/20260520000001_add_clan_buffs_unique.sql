DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'clan_buffs_clan_id_buff_name_key'
    ) THEN
        ALTER TABLE clan_buffs ADD CONSTRAINT clan_buffs_clan_id_buff_name_key UNIQUE (clan_id, buff_name);
    END IF;
END $$;
