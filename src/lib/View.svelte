<script>
  document.oncontextmenu = function () { return false; }
  window.onresize = function() {
      let width = document.querySelector('.ikusei div').clientWidth;
      document.querySelectorAll('.ikusei_subchart canvas')[0].style.width = width + 'px';
      document.querySelectorAll('.ikusei_subchart canvas')[1].style.width = width + 'px';
  }

  import { invoke } from "@tauri-apps/api/tauri"
  import { Command } from '@tauri-apps/api/shell'
  import { convertFileSrc } from '@tauri-apps/api/tauri';
  import { listen, emit } from '@tauri-apps/api/event';

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
  } from '@smui/data-table';
  import ImageList, { Item, Image } from '@smui/image-list';
  import Paper, { Title, Content } from '@smui/paper';

  import { Line, PolarArea, Bar } from 'svelte-chartjs';
  import {
      Chart as ChartJS,
      Title as Ctitle,
      Tooltip as Ctip,
      Legend,
      BarElement,
      LineElement,
      LinearScale,
      PointElement,
      CategoryScale,
      ArcElement,
      RadialLinearScale,
  } from 'chart.js';

  ChartJS.register(
      Ctitle,
      Ctip,
      Legend,
      BarElement,
      LineElement,
      LinearScale,
      PointElement,
      CategoryScale,
      ArcElement,
      RadialLinearScale
  );

  let loggingMsg = ["Start をクリックするとロギングを開始します"];
  let filename = getFilename();
  let process = null;
  let iid = false;
  let logno = "0";
  let active = "Console";
  let items = [];
  let startStats = '';
  let stopStats = "disabled";
  let canScreenshot = "";

  function getFilename() {
    let now = new Date();
    return String(now.getFullYear()) + String(now.getMonth()+1).padStart(2, '0') + String(now.getDate()).padStart(2, '0') + String(now.getHours()).padStart(2, '0') + String(now.getMinutes()).padStart(2, '0') + String(now.getSeconds()).padStart(2, '0') + ".tsv"
  }

  function addMsg(m) {
    if(m == '') { return; }

    let tmp = loggingMsg.concat();
    tmp.push(m);
    loggingMsg = tmp;
  }

  async function startLog() {
    if(process) { return; }

    let path = await invoke("get_path");
    let cmd = new Command('umalog', [
      path + "\\out\\" + filename
    ]);
    process = await cmd.spawn();
    addMsg("ロギングを開始しました。プロセスは " + process.pid + " です");
    startStats = "disabled";
    stopStats = "";

    let tid;
    tid = setTimeout(function() {
      console.log("getlog start");
      get_loglists(true);
      clearTimeout(tid);
    }, 1000);
  //  iid = setInterval(get_stdoutfile, 2000);

    listen('logrefresh', function(ret) {
      log_refresh(ret.payload);
    });
    emit('logcheck', { lognoStr: String(logno), filename: filename });
    iid = true;
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

  async function log_refresh(ret) {
    if(ret) {
      let parse = ret.split("\n");
      if(parse.length > 0) {
        logno = String(parse[0]);

        for(let i = 1, size = parse.length; i < size; i++) {
          addMsg(parse[i].replace("\t", ", "));
          
          if(String(parse[i]).indexOf('育成完了') !== -1) {
            stopLog();
            let tid;
            tid = setTimeout(function() {
              filename = getFilename();
              clearTimeout(tid);
            }, 2000);
            break;
          }
        }

        drowChart(filename);
      }
    }

    if(iid) {
      emit('logcheck', { lognoStr: String(logno), filename: filename });
    }
  }

  async function stopLog(silent) {
//    clearInterval(iid);
    iid = false;
    let cmd = new Command('taskkill', ["/im", "umalog.exe", "/f"]);
    process = await cmd.spawn();
    process = false;
    logno = "0";
    startStats = "";
    stopStats = "disabled";
    
    if(silent == true ) { return; }

    addMsg("ロギングを停止しました");
  }
  stopLog(true);

  async function get_loglists(force) {
    if(active != 'List' && force !== true) { return; }
console.log('get_loglist');
    items = [];
    let ret = await invoke("get_loglists");
    let lists = JSON.parse(ret);
    let temp = [];

    if(!lists) { return; }
    console.log(lists);
    for(let i in lists) {
      let dt = new Date(lists[i].create_timestamp * 1000);
      let line = {
        filename: lists[i].filename,
        create_date: dt.getFullYear() + '/' + String((dt.getMonth()+1)).padStart(2, '0') + '/' + String(dt.getDate()).padStart(2, '0') + ' ' + String(dt.getHours()).padStart(2, '0') + ':' + String(dt.getMinutes()).padStart(2, '0')
      };
      temp.push(line);
    }

    temp.reverse();
    items = temp;
  }
  get_loglists(true);

  async function take_screenshot() {
    canScreenshot = "disabled";
    let ret = await invoke("take_screenshot");
    canScreenshot = "";
    if(ret) {
      get_imagelist();
    }
  }

  async function imageview(img) {
    let path = await invoke("get_path");
    let cmd = new Command('view', ["/c", "start", path + "\\screenshot\\" + img]);
    cmd.spawn();
  }

  let imagelist = [];
  async function get_imagelist() {
    invoke("get_imagelist").then(function(ret) {
      let images = JSON.parse(ret);
      images.reverse();

      imagelist = [];
      imagelist = images;
    });
  }
  let tid;
  tid = setTimeout(function() {
    get_imagelist();
  }, 1000);

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

  // グラフ初期値

  let maxScale = 1600;

  let chartIkusei = {
    labels: ['ジュニア級テビュー前'],
    datasets: [{
      label: 'スピード',
      data: [100],
      borderColor: '#1565C0',
      backgroundColor: '#42A5F5',
      pointStyle: 'circle',
      pointHoverRadius: 10
    }, {
      label: 'スタミナ',
      data: [100],
      borderColor: '#C62828',
      backgroundColor: '#EF5350',
      pointStyle: 'circle',
      pointHoverRadius: 10
    }, {
      label: 'パワー',
      data: [100],
      borderColor: '#F9A825',
      backgroundColor: '#FFEE58',
      pointStyle: 'circle',
      pointHoverRadius: 10
    }, {
      label: '根性',
      data: [100],
      borderColor: '#6A1B9A',
      backgroundColor: '#AB47BC',
      pointStyle: 'circle',
      pointHoverRadius: 10
    }, {
      label: '賢さ',
      data: [100],
      borderColor: '#558B2F',
      backgroundColor: '#9CCC65',
      pointStyle: 'circle',
      pointHoverRadius: 10
    }, {
      label: 'スキルPt',
      data: [120],
      borderColor: '#424242',
      backgroundColor: '#BDBDBD',
      pointStyle: 'circle',
      pointHoverRadius: 10
    }]
  };

  let chartResult = {
    labels: ['スピード', 'スタミナ', 'パワー', '根性', '賢さ'],
    datasets: [{
      data: [100, 100, 100, 100, 100],
      backgroundColor: [ '#42A5F5', '#EF5350', '#FFEE58', '#AB47BC', '#9CCC65' ],
      pointRadius: 8,
      pointHoverRadius: 10
    }]
  };

  let chartYaruki = {
    labels: ['絶好調', '好調', '普通', '不調', '絶不調', 'やる気ダウン'],
    datasets: [{
      label: '',
      data: [0, 0, 0, 0, 0, 0],
      backgroundColor: [
        'rgba(245, 127, 158, 0.7)',
        'rgba(245, 170, 65, 0.7)',
        'rgba(245, 214, 24, 0.7)',
        'rgba(15, 171, 245, 0.7)',
        'rgba(200, 128, 245, 0.7)',
        'rgba(100, 100, 100, 0.7)',
      ],
      borderWidth: 2,
      borderColor: [
        'rgba(245, 127, 158, 1)',
        'rgba(245, 170, 65, 1)',
        'rgba(245, 214, 24, 1)',
        'rgba(15, 171, 245, 1)',
        'rgba(200, 128, 245, 1)',
        'rgba(100, 100, 100, 1)',
      ]
    }]
  };

  // グラフ表示
  async function drowChart(filename) {
    let ret = await invoke("get_filelog", {filename});
    let data = ret.split("\n");
    let labels = []; // csvの0列がラベル（シーズン、X軸）
    let speeds = []; // スピード csv[1]
    let staminas = []; // スタミナ csv[2]
    let powers = []; // パワー csv[3]
    let mentals = []; // 根性 csv[4]
    let intellis = []; // 賢さ csv[5]
    let skillpts = []; // スキルPt csv[6]
    let yarukis = []; // やる気 csv[7]

    for(let i in data) {
      if(Number(i) == 0) { continue; }
      if(!data[i]) { continue; }

      let csv = data[i].split("\t");
      labels.push(csv[0].replace(/[^クラシック|ジュニア|シニア|開催中|育成完了]/, "").replace("ジュニア", "Jr.").replace("クラシック", "Cl.").replace("シニア", "Sr."));
      speeds.push(csv[1]);
      staminas.push(csv[2]);
      powers.push(csv[3]);
      mentals.push(csv[4]);
      intellis.push(csv[5]);
      skillpts.push(csv[6]);
      if(csv[7]) {
        yarukis.push(csv[7]);
      }
    }

    let ikuseiVal = [{
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
    }];

    if(JSON.stringify(ikuseiVal) != JSON.stringify(chartIkusei.datasets)) {
      chartIkusei = {
        labels: labels,
        datasets: ikuseiVal
      };
    }

    let radarVal = [100, 100, 100, 100, 100];

    if(speeds.length > 0) {
      radarVal[0] = Number(speeds[speeds.length-1]);
      radarVal[1] = Number(staminas[staminas.length-1]);
      radarVal[2] = Number(powers[powers.length-1]);
      radarVal[3] = Number(mentals[mentals.length-1]);
      radarVal[4] = Number(intellis[intellis.length-1]);
      //maxScale = Math.max.apply(null, radarVal) + 100;
    }

    chartResult.datasets[0].data = radarVal;

    //if(maxScale <= 700) {
    //  maxScale = 1200;
    //}

    if(yarukis) {
      let yarukidata = [0, 0, 0, 0, 0, 0];
      let m = { '絶好調': 4, '好調': 3, '普通': 2, '不調': 1, '絶不調': 0 };
      let past = 2;

      for(let i in yarukis) {
        let y = yarukis[i];
        let n = m[y];

        if(y == '絶好調') {
          yarukidata[0] = yarukidata[0]+=1;
        } else if(y == '好調') {
          yarukidata[1] = yarukidata[1]+=1;
        } else if(y == '普通') {
          yarukidata[2] = yarukidata[2]+=1;
        } else if(y == '不調') {
          yarukidata[3] = yarukidata[3]+=1;
        } else if(y == '絶不調') {
          yarukidata[4] = yarukidata[4]+=1;
        }

        if(n < past) {
          yarukidata[5] = yarukidata[5]+=1;
        }

        past = n;
      }

      chartYaruki.datasets[0].data = yarukidata;
    }

  }

</script>

<header>
  <h1>UmaUmaLogger</h1>
  <div>
    <Textfield variant="outlined" bind:value={filename} label="Filename"></Textfield>
  </div>
  <div>
    <Wrapper>
      <Button on:click={startLog} disabled="{startStats}">
        <Icon class="fa-solid fa-play"></Icon>
      </Button>
      <Tooltip>ロギングを開始します</Tooltip>
    </Wrapper>
    <Wrapper>
      <Button on:click={stopLog} disabled="{stopStats}">
        <Icon class="fa-solid fa-stop"></Icon>
      </Button>
      <Tooltip>ロギングを停止します</Tooltip>
    </Wrapper>
    <Wrapper>
      <Button on:click={take_screenshot} disabled="{canScreenshot}">
        <Icon class="fa-solid fa-image"></Icon>
      </Button>
      <Tooltip>スクリーンショットを撮ります</Tooltip>
    </Wrapper>
  </div>
</header>

<section>
<div class="ikussei_chart">
  <Line
      data={chartIkusei}
      height={260}
      options={{ maintainAspectRatio: false, responsive: true }}
  />
</div>

<div class="ikusei"><div></div><div></div></div>
<div class="ikusei">
  <div class="ikusei_log">
    <TabBar tabs={['Console', 'Logs', 'Events']} let:tab bind:active>
      <Tab {tab} minWidth>
        <Label>{#if tab == 'Console'}コンソール{:else if tab == 'Logs'}ログリスト{:else if tab == 'Events'}イベントチェック{/if}</Label>
      </Tab>
    </TabBar>

    {#if active == 'Console'}
      <div class="console">
        <ul>
        {#each loggingMsg as msg}
          <li>{msg}</li>
        {/each}
        </ul>
      </div>
    {:else if active == 'Logs' }
      <div class="logs">
        <DataTable
          table$aria-label="Log list"
          style="width: 100%;"
        >
        <Head>
          <Row>
            <Cell columnId="filename" style="width: 100%;">
              <Label>File Name</Label>
            </Cell>
            <Cell columnId="create_date">
              <Label>Create Date</Label>
            </Cell>
          </Row>
        </Head>
        <Body>
          {#each items as item }
            <Row>
              <Cell on:click={() => drowChart(item.filename)}>{item.filename}</Cell>
              <Cell on:click={() => drowChart(item.filename)}>{item.create_date}</Cell>
            </Row>
          {/each}
        </Body>
        </DataTable>
      </div>
    {:else if active == 'Events'}
      <div class="events">
        <Textfield bind:value={event_name} label="イベント名"></Textfield>
        <Textfield bind:value={musumename} label="育成ウマ娘"></Textfield>
        <Wrapper>
          <Button on:click={check_events} variant="raised">
            <Icon class="fas fa-solid fa-magnifying-glass"></Icon>
          </Button>
          <Tooltip>表示中のイベントをチェックします</Tooltip>
        </Wrapper>
        <Title>{eventName}</Title>
        {#each events as ev}
          <Paper variant="unelevated">
            <Title>{ev.select}</Title>
            <Content>{ev.value}</Content>
          </Paper>
          <hr />
        {/each}
      </div>
    {/if}
  </div>

  <div class="ikusei_subchart">
      <div>
          <PolarArea 
              data={chartResult}
              options={{
                maintainAspectRatio: false,
                responsive: true,
                plugins: {
                  legend: {
                    display: false
                  }
                },
                scale: {
                  beginAtZero: true,
                  max: maxScale,
                  min: 0,
                  stepSize: 100
                }
              }}
          />
      </div>

      <div>
          <Bar 
              data={chartYaruki}
              options={{
                maintainAspectRatio: false,
                responsive: true,
                indexAxis: 'y',
                plugins: {
                  legend: {
                    display: false
                  }
                }
              }}
          />
      </div>
  </div>
</div>
</section>

<section>
<div class="screenshot">
  <Group variant="raised">
    <Wrapper>
      <Button on:click={take_screenshot} variant="raised" disabled="{canScreenshot}">
        <Icon class="fa-solid fa-image"></Icon>
        <Label>Capture</Label>
      </Button>
      <Tooltip>スクリーンショットを撮ります</Tooltip>
    </Wrapper>
    <Wrapper>
      <Button on:click={show_screenshotdir} variant="raised">
        <Icon class="fa-regular fa-folder-open"></Icon>
      </Button>
      <Tooltip>スクリーンショットフォルダを開きます</Tooltip>
    </Wrapper>
  </Group>

  <div class="screenshots">
  <ImageList>
    {#each imagelist as img, i}
      <Item on:click={imageview(img.filename)}>
        <Image
          src="{convertFileSrc("screenshot/" + img.filename)}"
        />
      </Item>
    {/each}
  </ImageList>
  </div>
</div>
</section>
