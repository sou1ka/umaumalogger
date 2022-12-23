<script>
    import Tab from '@smui/tab';
    import TabBar from '@smui/tab-bar';
    import Label from '@smui/button';
    import Console from './lib/Console.svelte';
    import Logs from './lib/Logs.svelte';
    import Events from './lib/Events.svelte';
    import Screenshot from './lib/Screenshot.svelte';

    import IconButton from '@smui/icon-button';
    import Dialog, { Content } from '@smui/dialog';

    let active = "Console";
    let items = [];
    let open = false;
  </script>
  
  <main class="container">
    <h1>UmaUmaLogger</h1>
  
    <div class="row">
        <div class="tab">
            <TabBar tabs={['Console', 'List', 'Events', 'Screenshot']} let:tab bind:active>
                <Tab {tab} minWidth>
                <Label>{tab}</Label>
                </Tab>
            </TabBar>
        </div>

        {#if active === 'Console'}
            <Console />
        {:else if active === 'List'}
            <Logs items={items} open={open} />
        {:else if active === 'Events'}
            <Events />
        {:else if active === 'Screenshot'}
            <Screenshot />
        {/if}
    </div>
    
    <Dialog bind:open sheet aria-describedby="sheet-content">
    <Content id="sheet-content">
        <IconButton action="close" class="fa-solid fa-circle-xmark"></IconButton>
        <canvas class="chart" id="chart_ikusei"></canvas>
        <canvas class="chart" id="chart_result"></canvas>
    </Content>
    </Dialog>

  </main>
  
  <style>
  </style>
  