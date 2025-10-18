<script lang="ts">
  import Chart, { type ChartConfiguration } from 'chart.js/auto'
  import 'hw-chartjs-plugin-colorschemes';
  import { Classic10 } from 'hw-chartjs-plugin-colorschemes/src/colorschemes/colorschemes.tableau';
  import 'moment';
  import 'chartjs-adapter-moment';
  import RealTimeScale from '@robloche/chartjs-plugin-streaming';
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AppLayout from "../AppLayout.svelte";
  import { authStore } from "$lib/stores/authStore";
  import Choices, {type InputChoice } from "choices.js";
  import "choices.js/public/assets/styles/choices.css";

  interface InterfaceData {
    name: string;
    "bytes received": string;
    "bytes transmitted": string;
    device: string;
    driver: string;
    color: String;
  }

  interface InterfaceTraffic {
    interfaces: Record<string, InterfaceData>;
    time: number;
    interval: number;
  }

  interface InterfaceTopData {
      records: InterfaceTopRecord[];
      status: number;
  }

  interface InterfaceTopRecord {
      address: String;
      rate_bits_in: number;
      rate_bits_out: number;
      rate_bits: number;
      cumulative_bytes_in: number;
      cumulative_bytes_out: number;
      cumulative_bytes: number;
      tags: String[];
      rname: String;
      rate_in: String;
      rate_out: String;
      rate: String;
      cumulative_in: String;
      cumulative_out: String;
      cumulative: String;
  }

  type TopTrafficRecords = Record<string, InterfaceTopData> | null;
  
  Chart.register(RealTimeScale);

  let test: HTMLElement;
  let interfaceChoice: HTMLElement;
  let rxChart: HTMLElement;
  let txChart: HTMLElement;
  let rxTopChart: HTMLElement;
  let txTopChart: HTMLElement;

  // For interface filtering
  let startPolling = false;
  let interfaces: InputChoice[] | null = null;
  let selectedInterfaceOptions: string[] = [];

  function set_alpha(color: string, opacity: number) {
    const op = Math.round(Math.min(Math.max(opacity || 1, 0), 1) * 255);
    return color + op.toString(16).toUpperCase();
  }

  function format_field(value: number) {
    if (!isNaN(value) && value > 0) {
      let fileSizeTypes = ["", "K", "M", "G", "T", "P", "E", "Z", "Y"];
      let ndx = Math.floor(Math.log(value) / Math.log(1000) );
      if (ndx > 0) {
        return  (value / Math.pow(1000, ndx)).toFixed(2) + ' ' + fileSizeTypes[ndx];
      } else {
        return value.toFixed(2);
      }
    } else {
      return "";
    }
  }

  onMount(async () => {
    const interfaceChoices = new Choices(interfaceChoice, {
      duplicateItemsAllowed: false,
      searchEnabled: false,
      searchChoices: false,
      removeItemButton: true,
      placeholderValue: 'Select an interface',
      callbackOnCreateTemplates: function(strToEl, escapeForTemplate, getClassNames) {
        return {
          item: (
            options: any,
            choice: any,
            removeItemButton: boolean,
          ) => {
            let template = Choices.defaults.templates.item(options, choice, removeItemButton);
            template.style.backgroundColor = choice.customProperties.color;
            return template;
          },
        }
      },
    });

    if ($authStore.isLoggedIn) {
      /**
       * create new traffic chart
       */
      function traffic_graph(target: any, graph_title: string, init_data: any): Chart {
        // setup legend
        let all_datasets: any[] = [];
        Object.keys(init_data.interfaces).forEach(function(intf) {
          all_datasets.push({
            label: init_data.interfaces[intf].name,
            hidden: true,
            borderColor: init_data.interfaces[intf].color,
            backgroundColor: set_alpha(init_data.interfaces[intf].color, 0.5),
            pointHoverBackgroundColor: init_data.interfaces[intf].color,
            pointHoverBorderColor: init_data.interfaces[intf].color,
            pointBackgroundColor: init_data.interfaces[intf].color,
            pointBorderColor: init_data.interfaces[intf].color,
            intf: intf,
            last_time: init_data.time,
            last_data: init_data.interfaces[intf][target.dataset.src_field],
            src_field: target.dataset.src_field,
            data: []
          });
        });
        // new chart
        var ctx = target.getContext('2d');
        var config: ChartConfiguration = {
          type: 'line',
          data: {
            datasets: all_datasets
          },
          options: {
            maintainAspectRatio: false,
            elements: {
              line: {
                fill: true,
                cubicInterpolationMode: 'monotone',
                clip: 0
              }
            },
            scales: {
              x: {
                time: {
                  tooltipFormat:'HH:mm:ss',
                  unit: 'second',
                  stepSize: 10,
                  minUnit: 'second',
                  displayFormats: {
                    second: 'HH:mm:ss',
                    minute: 'HH:mm:ss'
                  }
                },
                type: 'realtime',
                realtime: {
                  duration: 20000,
                  refresh: init_data.interval,
                  delay: init_data.interval
                },
                grid: {
                  color: getComputedStyle(document.documentElement).getPropertyValue('--chart-grid-color'),
                },
              },
              y: {
                ticks: {
                  callback: function (value: any, index: any, values: any) {
                    return format_field(value);
                  }
                },
                grid: {
                  color: getComputedStyle(document.documentElement).getPropertyValue('--chart-grid-color'),
                },
              }
            },
            hover: {
              mode: 'nearest',
              intersect: false
            },
            plugins: {
              tooltip: {
                mode: 'nearest',
                intersect: false,
                callbacks: {
                  label: function(context: any) {
                    return context.dataset.label + ": " + format_field(context.dataset.data[context.dataIndex].y).toString();
                  }
                }
              },
              title: {
                display: true,
                text: graph_title
              },
              legend: {
                display: false
              },
              streaming: {
                frameRate: 30
              },
              colorschemes: {
                scheme: 'tableau.Classic10'
              }
            }
          }
        };

        let chart = new Chart(ctx, config)
        document.documentElement.addEventListener('onToggleTheme', (e: Event) => {
          chart.options.scales!.x!.grid!.color = getComputedStyle(document.documentElement).getPropertyValue('--chart-grid-color');
          chart.options.scales!.y!.grid!.color = getComputedStyle(document.documentElement).getPropertyValue('--chart-grid-color');
          chart.update();
        });
        return chart;
      }
      /**
       * create new traffic top usage chart
       */
      function traffic_top_graph(target: any, graph_title: string, init_data: any): Chart {
        // setup legend
        let all_datasets: any = [];
        Object.keys(init_data.interfaces).forEach(function(intf) {
          all_datasets.push({
            label: init_data.interfaces[intf].name,
            hidden: true,
            borderColor: init_data.interfaces[intf].color,
            backgroundColor: set_alpha(init_data.interfaces[intf].color, 0.5),
            pointHoverBackgroundColor: init_data.interfaces[intf].color,
            pointHoverBorderColor: init_data.interfaces[intf].color,
            pointBackgroundColor: init_data.interfaces[intf].color,
            pointBorderColor: init_data.interfaces[intf].color,
            intf: intf,
            last_time: init_data.time,
            last_data: init_data.interfaces[intf][target.dataset.src_field],
            src_field: target.dataset.src_field,
            data: []
          });
        });
        // new chart
        var ctx = target.getContext('2d');
        var config: ChartConfiguration = {
          type: 'bubble',
          data: {
            datasets: all_datasets
          },
          options: {
            maintainAspectRatio: false,
            scales: {
              x: {
                time: {
                  tooltipFormat:'HH:mm:ss',
                  unit: 'second',
                  //stepSize: init_data.interval < 10000 ? 5 : init_data.interval / 1000,
                  stepSize: 10,
                  minUnit: 'second',
                  displayFormats: {
                    second: 'HH:mm:ss',
                    minute: 'HH:mm:ss'
                  }
                },
                type: 'realtime',
                realtime: {
                  duration: 40000,
                  refresh: init_data.interval,
                  delay: init_data.interval,
                },
                grid: {
                  color: getComputedStyle(document.documentElement).getPropertyValue('--chart-grid-color'),
                },
              },
              y: {
                ticks: {
                  callback: function (value: any, index: any, values: any) {
                    return format_field(value);
                  }
                },
                grid: {
                  color: getComputedStyle(document.documentElement).getPropertyValue('--chart-grid-color'),
                },
              }
            },
            hover: {
              mode: 'nearest',
              intersect: false
            },
            plugins: {
              tooltip: {
              mode: 'nearest',
              intersect: false,
              callbacks: {
                label: function(context: any) {
                  let split = context.formattedValue.split(",")[0]
                  let time = split.replace('(', '')
                  return [
                    time,
                    context.dataset.label + ": " + (context.dataset.data[context.dataIndex].rname && context.dataset.data[context.dataIndex].rname != '' ? context.dataset.data[context.dataIndex].rname : context.dataset.data[context.dataIndex].address),
                    "@ " + format_field(context.dataset.data[context.dataIndex].y).toString()
                  ];
                }
              }
              },
              title: {
                display: true,
                text: graph_title
              },
              legend: {
                display: false,
              },
              streaming: {
                frameRate: 30
              },
              colorschemes: {
                scheme: 'tableau.Classic10'
              }
            }
          }
        };
        
        let chart = new Chart(ctx, config)
        document.documentElement.addEventListener('onToggleTheme', (e: Event) => {
          chart.options.scales!.x!.grid!.color = getComputedStyle(document.documentElement).getPropertyValue('--chart-grid-color');
          chart.options.scales!.y!.grid!.color = getComputedStyle(document.documentElement).getPropertyValue('--chart-grid-color');
          chart.update();
        });
        return chart;
      }

      function update_traffic_charts(charts: any, data: any) {
        charts.forEach(function(chart: any) {
          Object.keys(data.interfaces).forEach(function(intf) {
            chart.config.data.datasets.forEach(function(dataset: any) {
              if (dataset.intf == intf) {
                let calc_data = data.interfaces[intf][dataset.src_field];
                let elapsed_time = data.time - dataset.last_time;
                // dataset.hidden = !$("#interfaces").val().includes(intf);
                dataset.hidden = selectedInterfaceOptions.includes(intf) ? false : true;
                dataset.data.push({
                  x: Date.now(),
                  y: Math.round(((calc_data - dataset.last_data) / elapsed_time) * 8)
                });
                dataset.last_time = data.time;
                dataset.last_data = calc_data;
                return;
              }
            });
          });
          chart.update('quiet');
        });
      }
      
      function update_top_charts(charts: any, data: any) {
        charts.forEach(function(chart: any) {
          Object.keys(data).forEach(function(intf) {
            chart.config.data.datasets.forEach(function(dataset:any) {
              if (dataset.intf == intf) {
                let calc_data = data[intf]['records'];
                //dataset.hidden = !$("#interfaces").val().includes(intf);
                dataset.hidden = false;
                for (var i=0; i < data[intf]['records'].length ; ++i) {
                  dataset.data.push({
                    "x": Date.now(),
                    "y": data[intf]['records'][i]['rate_bits_' + dataset.src_field],
                    "r": 4,
                    "address": data[intf]['records'][i]['address'],
                    // ADDED/UPDATED
                    "rname": data[intf]['records'][i]['rname']
                  });
                }
                return;
              }
            });
          });
          chart.update('quiet');
        });
      }

      // Store references to the charts globally
      let g_charts = {traffic: [] as Chart[], traffic_top: [] as Chart[], interval: 1000};

      loadInterfaceList().then((data) => {
        if (!data) return;
        
        interfaces = [];

        let i = 0;
        Object.keys(data.interfaces).forEach(function(intf) {
          if(intf == "lan" || intf == "wan")
            selectedInterfaceOptions.push(intf);

          let colors = Classic10;
          let colorIdx = i % colors.length;
          data.interfaces[intf].color = colors[colorIdx];

          interfaces?.push({
            value: intf, 
            label: data.interfaces[intf].name,
            selected: intf == "lan" || intf == "wan",
            customProperties: {
              color: colors[colorIdx]
            }
          } as InputChoice);

          i++;
        });

        interfaceChoices.setChoices(interfaces);

        data.interval = 1000;
        g_charts['interval'] = data.interval;

        const chart_types = [rxChart, txChart, rxTopChart, txTopChart];
        chart_types.forEach(function(chart) {
          /* Create the charts */
          let graph: Chart;
          if (chart.id.includes('Top')) {
            let rxtx = chart.id.includes('rx') ? 'Top hosts in (bps)' : 'Top hosts out (bps)';
            graph = traffic_top_graph(chart, rxtx, data);
            g_charts['traffic_top'].push(graph);
          }
          else {
            let rxtx = chart.id.includes('rx') ? 'In (bps)' : 'Out (bps)';
            graph = traffic_graph(chart, rxtx, data);
            g_charts['traffic'].push(graph);
          }
        });

        startPolling = true;
        /**
         * poll for new stats and update selected charts
         */
        (function traffic_poller() {
          loadInterfaceList()
          .then(data => {
            if (data?.interfaces !== undefined) {
              update_traffic_charts(g_charts['traffic'], data);
            }
          });
          setTimeout(traffic_poller, g_charts['interval']);
        })();

        (function top_traffic_poller() {
          if (!startPolling) return;
            loadInterfaceTopList()
            .then(data => {
              update_top_charts(g_charts['traffic_top'], data);
              top_traffic_poller();
            })
            .catch(error => {
              setTimeout(top_traffic_poller, g_charts['interval']);
            });
        })();
      })
      .catch(error => {
          console.error('Error:', error);
      });
    }
  });

  onDestroy(() => {
    startPolling = false;
  });

  function loadInterfaceList() {
    try {
      return invoke<InterfaceTraffic>("get_interface_traffic");
    } catch (err) {
      console.error("Failed to load interface list:", err);
      throw new Error("Failed to load interface list:", { cause: err });
    }
  }

  function loadInterfaceTopList() {
    if (selectedInterfaceOptions.length == 0) {
      throw new Error("No interfaces selected.");
    }

    try {
      return invoke<TopTrafficRecords>("get_interface_top_traffic", {interfaces: selectedInterfaceOptions});
    } catch (err) {
      console.error("Failed to load interface top traffic list:", err);
      throw new Error("Failed to load interface top traffic list:", { cause: err });
    }
  }
</script>
<AppLayout>
  <style>
    [data-theme="dark"] .choices__inner {
      background-color: #191e24;
      border: 1px solid #3b3e40;
    }
    [data-theme="dark"] .choices__input {
      background-color: #191e24;
    }
    [data-theme="dark"] .choices__item--choice {
      background-color: #191e24 !important;
    }
    /* Light Theme */
    :root {
      --chart-grid-color: #e5e5e5;
    }
    /* Dark Theme */
    :root[data-theme="dark"] {
      --chart-grid-color: #3b3e40;
    }
  </style>
  <div class="max-w-6xl mx-auto">
    <h2 class="text-2xl font-bold mb-6">Traffic</h2>
    <div class="bg-base-100 rounded-lg shadow p-4 mb-4">
      <div>
          <select bind:this={interfaceChoice} id="interfaces" name="states[]" multiple bind:value={selectedInterfaceOptions}>
          </select>
      </div>
    </div>
    <div class="bg-base-100 rounded-lg shadow p-4 mb-4">
      <div>
        <canvas bind:this={rxChart} id="rxChart" data-src_field="bytes received"></canvas>
      </div>
      <div>
        <canvas bind:this={txChart} id="txChart" data-src_field="bytes transmitted"></canvas>
      </div>
      <div>
        <canvas bind:this={rxTopChart} id="rxTopChart" data-src_field="in"></canvas>
      </div>
      <div>
        <canvas bind:this={txTopChart} id="txTopChart" data-src_field="out"></canvas>
      </div>
    </div>
  </div>
</AppLayout>
