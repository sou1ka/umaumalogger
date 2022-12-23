<script>
    import { invoke } from "@tauri-apps/api/tauri"
    import { listen } from '@tauri-apps/api/event';

    import Textfield from '@smui/textfield';
    import Tooltip, { Wrapper } from '@smui/tooltip';
    import Button, { Label, Icon } from '@smui/button';
    import { Title, Content } from '@smui/dialog';
    import Paper from '@smui/paper';

    let musumename = '';
    let eventName = '';
    let event_name = '';
    let events = [];

    async function check_events() {
        let ret = await invoke("get_eventvalue", {
            musumename: musumename,
            eventname: event_name,
            force: true
        });

        event_refresh(ret);
    }

    async function event_refresh(ret) {
        if(!ret) { return; }

        let event = JSON.parse(ret);
        eventName = event.eventName;
        events = event.events;
    }

    listen('eventrefresh', function(ret) {
        event_refresh(ret.payload);
    });
</script>


<div class="row">
    <Textfield variant="outlined" bind:value={event_name} label="イベント名"></Textfield>
    <Textfield variant="outlined" bind:value={musumename} label="育成ウマ娘"></Textfield>
    <Wrapper>
        <Button on:click={check_events} variant="raised">
        <Icon class="fas fa-light fa-list-check"></Icon>
        <Label>Check</Label>
        </Button>
        <Tooltip>表示中のイベントをチェックします</Tooltip>
    </Wrapper>
</div>

<div class="paper-container">
    <Title>{eventName}</Title>
    {#each events as ev}
        <Paper variant="unelevated">
        <Title>{ev.select}</Title>
        <Content>{ev.value}</Content>
        </Paper>
        <hr />
    {/each}
</div>