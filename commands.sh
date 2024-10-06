#!/bin/sh
# fail the command if env DISCORD_TOKEN is not set
if [ -z "$DISCORD_TOKEN" ]; then
  echo "DISCORD_TOKEN is not set"
  exit 1
fi

if [ -z "$DISCORD_APPLICATION_ID" ]; then
  echo "DISCORD_APPLICATION_ID is not set"
  exit 1
fi

curl -X PUT \
-H "Authorization: Bot $DISCORD_TOKEN" \
-H "Content-Type: application/json" \
-d @./commands.json \
"https://discord.com/api/v10/applications/$DISCORD_APPLICATION_ID/commands" | jq
