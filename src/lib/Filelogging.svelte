<script>
  import { invoke } from "@tauri-apps/api/tauri"
  import { Command } from '@tauri-apps/api/shell'
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

  let loggingMsg = ["Start をクリックするとロギングを開始します"];
  let now = new Date();
  let filename = String(now.getFullYear()) + String(now.getMonth()+1) + String(now.getDate()) + String(now.getHours()) + String(now.getMinutes()) + String(now.getSeconds()) + ".tsv"
  let stdoutfilename = filename + ".stdout.log";
  let process = null;
  let iid = null;

  function addMsg(m) {
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

    iid = setInterval(get_stdoutfile, 1000);
  }

  async function get_stdoutfile() {
    let runtime = (new Date().getTime() + 11644473600000) * 10000;
    let ret = await invoke("get_filelog_lastline", {
      runtimeStr: String(runtime), 
      filename: filename
    });

    if(ret) {
      addMsg(ret.replace("\t", ", "));
    }
  }

  async function stopLog() {
    clearInterval(iid);
    iid = null;

//    if(!process) { return; }

    let cmd = new Command('taskkill', ["/im", "umalog.exe", "/f"]);
    process = await cmd.spawn();
    process = false;
    addMsg("ロギングを停止しました");
  }

  async function get_loglists() {
    let ret = await invoke("get_loglists");
    let lists = JSON.parse(ret);
    
    if(!lists) { return; }

    for(let i in lists) {
      let dt = new Date(lists[i].create_timestamp * 1000);
      let line = {
        filename: lists[i].filename,
        create_date: dt.getFullYear() + '/' + (dt.getMonth()+1) + '/' + dt.getDate() + ' ' + dt.getHours() + ':' + dt.getMinutes()
      };
      items.push(line);
    }
  }
  get_loglists();

  // app setting
  let active = "Console";

  let items = [];
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
    /*  chartResult = new Chart(document.getElementById('chart_result'), {
        type: "radar",
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
      });*/
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
  }
</script>

<div class="tab">
  <TabBar tabs={['Console', 'List']} let:tab bind:active>
    <Tab {tab} minWidth>
      <Label>{tab}</Label>
    </Tab>
  </TabBar>
</div>

{#if active === 'Console'}
<div>
  <div class="row">
    <Textfield variant="outlined" bind:value={filename} label="Filename"></Textfield>
    <Group variant="raised">
      <Wrapper>
        <Button on:click={startLog} variant="raised">
          <Icon class="material-icons">fast_forward</Icon>
          <Label>Start</Label>
        </Button>
        <Tooltip>ロギングを開始します</Tooltip>
      </Wrapper>
      <Wrapper>
        <Button on:click={stopLog} variant="raised">
          <Label>Stop</Label>
          <Icon class="material-icons">close</Icon>
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
</div>

{:else if active === 'List'}

<div>
  <Wrapper>
    <Button on:click={get_loglists} variant="raised">
      <Icon class="material-icons">refresh</Icon>
      <Label>Reload</Label>
    </Button>
    <Tooltip>ログをリロードします</Tooltip>
  </Wrapper>
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
        <IconButton class="material-icons">arrow_upward</IconButton>
      </Cell>
      <Cell columnId="create_date">
        <Label>Create Date</Label>
        <IconButton class="material-icons">arrow_upward</IconButton>
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
{/if}

<Dialog bind:open sheet aria-describedby="sheet-content">
  <Content id="sheet-content">
    <IconButton action="close" class="material-icons">close</IconButton>
    <canvas class="chart" id="chart_ikusei"></canvas>
  </Content>
</Dialog>
