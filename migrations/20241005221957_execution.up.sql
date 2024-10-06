create table if not exists execution (
	id bigserial primary key,
	channel_id text not null,
	message_id text not null,
	reply_id text not null,
	language text not null,
	created_at timestamptz not null default now()
);

create index if not exists execution_message_id_index on execution (message_id);
