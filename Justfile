default:
    @just --list

serve *ARGS:
    npm run tauri:serve {{ARGS}}

build *ARGS:
    npm run tauri:build {{ARGS}}

dev:
    pane=$(tmux split-window -P -F '#{pane_id}' -h vue-cli-service serve); \
        trap "tmux kill-pane -t $pane" EXIT; \
        tauri dev --config '{ "build": { "devPath": "http://localhost:3000" }}'
