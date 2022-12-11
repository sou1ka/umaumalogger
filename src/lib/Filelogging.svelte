<script>
  import { invoke } from "@tauri-apps/api/tauri"
  import { Command } from '@tauri-apps/api/shell'
  import { convertFileSrc } from '@tauri-apps/api/tauri';
  import { listen, emit  } from '@tauri-apps/api/event'

  import Button, { Group, Label, Icon } from '@smui/button';
  import Textfield from '@smui/textfield';
  import Tooltip, { Wrapper } from '@smui/tooltip';
  import Tab from '@smui/tab';
  import TabBar from '@smui/tab-bar';
  import DataTable, {
    Head,
    Body,
    Row,
    Cell,
    SortValue,
  } from '@smui/data-table';
  import IconButton from '@smui/icon-button';
  import Dialog, { Header, Title, Content } from '@smui/dialog';
  import LayoutGrid from '@smui/layout-grid';
  import Paper, { Subtitle } from '@smui/paper';

  import { WebviewWindow } from '@tauri-apps/api/window';
  function viewWindow() {
    new WebviewWindow('graph', {
      url: 'src/graph.html'
    });
  }

  let loggingMsg = ["Start をクリックするとロギングを開始します"];
  let now = new Date();
  let filename = String(now.getFullYear()) + String(now.getMonth()+1).padStart(2, '0') + String(now.getDate()).padStart(2, '0') + String(now.getHours()).padStart(2, '0') + String(now.getMinutes()).padStart(2, '0') + String(now.getSeconds()).padStart(2, '0') + ".tsv"
  let process = null;
  let iid = null;
  let logno = "0";
  let active = "Console";
  let items = [];

  function addMsg(m) {
    if(m == '') { return; }

    let tmp = loggingMsg.concat();
    tmp.push(m);
    loggingMsg = tmp;
    
    let ti = setTimeout(function() {
      document.querySelector(".console li:last-child").scrollIntoView();
      clearTimeout(ti);
    }, 100);
  }

  async function startLog() {
    if(process) { return; }

    let path = await invoke("get_path");
    let cmd = new Command('umalog', [
      path + "\\out\\" + filename
    ]);
    process = await cmd.spawn();
    addMsg("ロギングを開始しました。プロセスは " + process.pid + " です");

    get_loglists(true);
    iid = setInterval(get_stdoutfile, 2000);
  }

  async function get_stdoutfile() {
    let ret = await invoke("get_filelog_lastline", {
      lognoStr: String(logno),
      filename: filename
    });

    if(ret) {
      let parse = ret.split("\n");
      if(parse.length > 0) {
        logno = String(parse[0]);

        for(let i = 1, size = parse.length; i < size; i++) {
          addMsg(parse[i].replace("\t", ", "));
        }
      }
    }
  }

  async function stopLog(silent) {
    clearInterval(iid);
    iid = null;

//    if(!process) { return; }

    let cmd = new Command('taskkill', ["/im", "umalog.exe", "/f"]);
    process = await cmd.spawn();
    process = false;
    
    if(silent == true ) { return; }

    addMsg("ロギングを停止しました");
  }
  stopLog(true);

  async function get_loglists(force) {
    if(active != 'List' && force !== true) { return; }

    items = [];
    let ret = await invoke("get_loglists");
    let lists = JSON.parse(ret);

    if(!lists) { return; }

    for(let i in lists) {
      let dt = new Date(lists[i].create_timestamp * 1000);
      let line = {
        filename: lists[i].filename,
        create_date: dt.getFullYear() + '/' + String((dt.getMonth()+1)).padStart(2, '0') + '/' + String(dt.getDate()).padStart(2, '0') + ' ' + String(dt.getHours()).padStart(2, '0') + ':' + String(dt.getMinutes()).padStart(2, '0')
      };
      items.push(line);
    }
  }
  get_loglists(true);

  let scid;
  async function take_screenshot() {
    let now = new Date();
    let filename = String(now.getFullYear()) + String(now.getMonth()+1).padStart(2, '0') + String(now.getDate()).padStart(2, '0') + String(now.getHours()).padStart(2, '0') + String(now.getMinutes()).padStart(2, '0') + String(now.getSeconds()).padStart(2, '0') + ".png"
    let path = await invoke("get_path");
    let cmd = new Command('capture', ["umamusume", path + "\\screenshot\\" + filename]);
    cmd.spawn();

    scid = setTimeout(get_imagelist, 5000);
  }

  async function imageview(img) {
    let path = await invoke("get_path");
    let cmd = new Command('view', ["/c", "start", path + "\\screenshot\\" + img]);
    cmd.spawn();
  }

  let imagelist = [];
  async function get_imagelist(force) {
    //if(force) { imagelist = []; }
    let ret = await invoke("get_imagelist");
    let images = JSON.parse(ret);

    if(true) {
      imagelist = [];
      imagelist = images;
    } else if(imagelist.length < images.length) {
      imagelist.push(images[images.length-1]);
    }

    clearTimeout(scid);
  }
  get_imagelist(true);

  async function show_screenshotdir() {
    let path = await invoke("get_path");
    let cmd = new Command('view', ["/c", "start", "explorer", path + "\\screenshot\\"]);
    cmd.spawn();
  }

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

  // app setting
  let sort = 'create_date';
  let sortDirection = 'ascending';
  function handleSort() {
    items.sort((a, b) => {
      const [aVal, bVal] = [a[sort], b[sort]][
        sortDirection === 'ascending' ? 'slice' : 'reverse'
      ]();
      if (typeof aVal === 'string' && typeof bVal === 'string') {
        return aVal.localeCompare(bVal);
      }
      return Number(aVal) - Number(bVal);
    });
    items = items;
  }

  let open = false;
  let chartIkusei;
  let chartResult;
  
  async function rowClick(filename) {
    if(!chartIkusei) {
      chartIkusei = new Chart(document.getElementById('chart_ikusei'), {
        type: "line",
        data: {
          type: 'line',
          data: {
            labels: ['1', '2'],
            datasets: [{
              label: 'dummy',
              data: [1, 2]
            }]
          }
        },
        options: {
          responsive: true
        }
      });
      chartResult = new Chart(document.getElementById('chart_result'), {
        type: "radar",
        data: {
          labels: ['スピード', 'スタミナ', 'パワー', '根性', '賢さ'],
          datasets: [{
            data: [100, 100, 100, 100, 100],
            borderColor: ["#1565C0", "#C62828", "#F9A825", "#6A1B9A", "#558B2F"],
            backgroundColor: "rgba(200, 20, 80, 0.4)",
            pointRadius: 10,
            pointHoverRadius: 20
          }]
        },
        options: {
          plugins: {
            legend: {
              display: false
            }
          },
          responsive: true,
          scale: {
            beginAtZero: true,
            max: 1200,
            min: 0,
            stepSize: 100
          }
        }
      });
    }

    open = true;
    let ret = await invoke("get_filelog", {filename});
    let data = ret.split("\n");
    let labels = []; // csvの0列がラベル（シーズン、X軸）
    let speeds = []; // スピード csv[1]
    let staminas = []; // スタミナ csv[2]
    let powers = []; // パワー csv[3]
    let mentals = []; // 根性 csv[4]
    let intellis = []; // 賢さ csv[5]
    let skillpts = []; // スキルPt csv[6]

    for(let i in data) {
      if(i == 0) { continue; }
      if(!data[i]) { continue; }

      let csv = data[i].split("\t");
      labels.push(csv[0]);
      speeds.push(csv[1]);
      staminas.push(csv[2]);
      powers.push(csv[3]);
      mentals.push(csv[4]);
      intellis.push(csv[5]);
      skillpts.push(csv[6]);
    }

    chartIkusei.data = {
      labels: labels,
      datasets: [{
        label: 'スピード',
        data: speeds,
        borderColor: '#1565C0',
        backgroundColor: '#42A5F5',
        pointStyle: 'circle',
        pointHoverRadius: 10
      }, {
        label: 'スタミナ',
        data: staminas,
        borderColor: '#C62828',
        backgroundColor: '#EF5350',
        pointStyle: 'circle',
        pointHoverRadius: 10
      }, {
        label: 'パワー',
        data: powers,
        borderColor: '#F9A825',
        backgroundColor: '#FFEE58',
        pointStyle: 'circle',
        pointHoverRadius: 10
      }, {
        label: '根性',
        data: mentals,
        borderColor: '#6A1B9A',
        backgroundColor: '#AB47BC',
        pointStyle: 'circle',
        pointHoverRadius: 10
      }, {
        label: '賢さ',
        data: intellis,
        borderColor: '#558B2F',
        backgroundColor: '#9CCC65',
        pointStyle: 'circle',
        pointHoverRadius: 10
      }, {
        label: 'スキルPt',
        data: skillpts,
        borderColor: '#424242',
        backgroundColor: '#BDBDBD',
        pointStyle: 'circle',
        pointHoverRadius: 10
      }]
    };
    chartIkusei.update();

    let radarVal = [];
    radarVal.push(Number(speeds[speeds.length-1]));
    radarVal.push(Number(staminas[staminas.length-1]));
    radarVal.push(Number(powers[powers.length-1]));
    radarVal.push(Number(mentals[mentals.length-1]));
    radarVal.push(Number(intellis[intellis.length-1]));
    chartResult.data.datasets[0].data = radarVal;
    chartResult.options.scale.max = Math.max.apply(null, radarVal);
    chartResult.update();
  }
</script>

<div class="tab">
  <TabBar tabs={['Console', 'List', 'Events', 'Screenshot']} let:tab bind:active>
    <Tab {tab} minWidth on:click={get_loglists}>
      <Label>{tab}</Label>
    </Tab>
  </TabBar>
</div>

{#if active === 'Console'}
<div class="row">
  <Textfield variant="outlined" bind:value={filename} label="Filename"></Textfield>
  <Group variant="raised">
    <Wrapper>
      <Button on:click={startLog} variant="raised">
        <Icon class="fa-solid fa-play"></Icon>
        <Label>Start</Label>
      </Button>
      <Tooltip>ロギングを開始します</Tooltip>
    </Wrapper>
    <Wrapper>
      <Button on:click={stopLog} variant="raised">
        <Label>Stop</Label>
        <Icon class="fa-solid fa-stop"></Icon>
      </Button>
      <Tooltip>ロギングを停止します</Tooltip>
  </Wrapper>
  </Group>
</div>

<!--
<p><small>育成ステータスをロギングします。
  シーズン、スピード、スタミナ、パワー、根性、賢さ、スキルPtを取得します。
  OCRで認識するので、文字によっては正しく取れないかもしれません（8が3になったりします）</small></p>
-->

<hr />

<div class="console">
  <ul>
  {#each loggingMsg as msg}
    <li>{msg}</li>
  {/each}
  </ul>
</div>

{:else if active === 'List'}

<div>
  <!--
  <Wrapper>
    <Button on:click={get_loglists} variant="raised">
      <Icon class="material-icons">refresh</Icon>
      <Label>Reload</Label>
    </Button>
    <Tooltip>ログをリロードします</Tooltip>
  </Wrapper>
  -->
  <hr />
  <DataTable
    sortable
    bind:sort
    bind:sortDirection
    on:SMUIDataTable:sorted={handleSort}
    table$aria-label="Log list"
    style="width: 100%;"
  >
  <Head>
    <Row>
      <Cell columnId="filename" style="width: 100%;">
        <Label>File Name</Label>
        <!-- For non-numeric columns, icon comes second. -->
        <IconButton class="fas fa-duotone fa-arrow-down"></IconButton>
      </Cell>
      <Cell columnId="create_date">
        <Label>Create Date</Label>
        <IconButton class="fas fa-duotone fa-arrow-up"></IconButton>
      </Cell>
    </Row>
  </Head>
  <Body>
    {#each items as item }
      <Row>
        <Cell on:click={rowClick(item.filename)}>{item.filename}</Cell>
        <Cell on:click={rowClick(item.filename)}>{item.create_date}</Cell>
      </Row>
    {/each}
  </Body>
  </DataTable>

</div>

{:else if active === 'Events'}

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

{:else if active === 'Screenshot'}
  <Group variant="raised">
    <Wrapper>
      <Button on:click={take_screenshot} variant="raised">
        <Icon class="fa-solid fa-image"></Icon>
        <Label>Capture</Label>
      </Button>
      <Tooltip>スクリーンショットを撮ります</Tooltip>
    </Wrapper>
    <Wrapper>
      <Button on:click={show_screenshotdir} variant="raised">
        <Label>Folder Open</Label>
        <Icon class="fa-regular fa-folder-open"></Icon>
      </Button>
      <Tooltip>スクリーンショットフォルダを開きます</Tooltip>
    </Wrapper>
  </Group>

  <hr />
  <div class="screenshots">
  <LayoutGrid>
    {#each imagelist as img, i}
      <Cell on:click={imageview(img.filename)}>
        <img src={convertFileSrc("screenshot/" + img.filename) } width="90" />
      </Cell>
    {/each}
  </LayoutGrid>
  </div>
{/if}

<Dialog bind:open sheet aria-describedby="sheet-content">
  <Content id="sheet-content">
    <IconButton action="close" class="fa-solid fa-circle-xmark"></IconButton>
    <canvas class="chart" id="chart_ikusei"></canvas>
    <canvas class="chart" id="chart_result"></canvas>
  </Content>
</Dialog>