<script context="module">
    import { invoke } from "@tauri-apps/api/tauri";

    import DataTable, {
        Head,
        Body,
        Row,
        Cell,
        SortValue,
    } from '@smui/data-table';
    import Label from '@smui/button';
    import IconButton from '@smui/icon-button';
    import Dialog, { Content } from '@smui/dialog';

    export let items;
    export let open;
    let chartIkusei;
    let chartResult;
    let sort = 'create_date';
    let sortDirection = 'ascending';

    export async function get_loglists() {
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
    get_loglists();

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

        let ret = await invoke("get_filelog", {filename});console.log(ret);
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
        <Cell on:click={() => (rowClick(item.filename))}>{item.filename}</Cell>
        <Cell on:click={() => (open = true)}>{item.create_date}</Cell>
    </Row>
    {/each}
</Body>
</DataTable>


<Dialog bind:open sheet aria-describedby="sheet-content">
<Content id="sheet-content">
    <IconButton action="close" class="fa-solid fa-circle-xmark"></IconButton>
    <canvas class="chart" id="chart_ikusei"></canvas>
    <canvas class="chart" id="chart_result"></canvas>
</Content>
</Dialog>