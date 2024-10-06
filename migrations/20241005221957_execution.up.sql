create table if not exists execution (
	id integer primary key autoincrement,
	channel_id text not null,
	message_id text not null,
	reply_id text not null,
	language text not null,
	created_at timestamp not null default current_timestamp
);

create index if not exists execution_message_id_index on execution (message_id);
