<script lang="ts">
  import moment from "moment";
  import { onMount } from "svelte";
  import CheckboxButton from "../comps/CheckboxButton.svelte";
  import { getLines, getPlayerJoinTimes } from "../retrieve-data";
  import { data, playerActiveTimes, rollingAverageSwitches, type RollingAverage } from "../stores";
  let origPlayer = "";
  let loading = 0;

  const defaultFrom = moment().subtract(1, "d").local(true);
  const defaultTo = moment().add(1, "m").local(true);

  let from = defaultFrom.toISOString(true).slice(0, 16);
  let to = defaultTo.toISOString(true).slice(0, 16);
  let player = "";
  $: player = player.trim()
  onMount(query);
  rollingAverageSwitches.subscribe(query)

  async function query() {
    loading++;
    try {
      origPlayer = player;
      const f = Math.floor((from ? moment(from) : defaultFrom).unix()/60);
      const t = Math.floor((to ? moment(to) : defaultTo).unix()/60);
      $data = await getLines(
        f, t,
        //player,
        Object.entries($rollingAverageSwitches).filter(([_, v]) => v)
        .map(([k, _]) => parseInt(k) as RollingAverage).sort()
      )
      $playerActiveTimes = player ? await getPlayerJoinTimes(f, t, player) : []
    } catch {}
    loading--;
  }
  const version = __APP_VERSION__;

  function getPlayerStats(times: [moment.Moment, moment.Moment][]): any | null {
    if (times.length == 0) return null;
    let totalTime = times.map(([f, t]) => moment.duration(t.diff(f))).reduce((acc, dur) => acc.add(dur)).humanize()
    let lastSession = times[times.length - 1];
    let lastLeft = lastSession === undefined ? "further back in time" : "on " + lastSession[1].toLocaleString();
    return {
      totalTime,
      from: from ? moment(from).toLocaleString() : "1 day ago",
      to: to ? moment(to).toLocaleString() : "now",
      lastLeft
    }
  }
  $: playerStats = getPlayerStats($playerActiveTimes)
</script>
<style lang="scss">
  #player-stats {
    color: #aaa;

    b {
      color: #fc0;
    }
  }

  * {
    user-select: none;
  }
</style>

<b style="font-size: 1.5em; color: #fc0">StatusTracker v{version}&nbsp;&nbsp;|&nbsp;&nbsp;</b>

<span>Show activity</span>
<label for="from">from </label><input type="datetime-local" id="from" bind:value={from}/>
<label for="to">to </label><input type="datetime-local" id="to" bind:value={to} />
<label for="player">for player </label><input type="text" id="player" bind:value={player} placeholder="username"/>
<button on:click={query}>Query</button><br>
{#if loading !== 0}
  <span id="player-stats">Loading...</span>
{:else if playerStats && player === origPlayer && player !== ""}
  <span id="player-stats"><b>{player}</b> has played
    for <b>{playerStats.totalTime}</b> between <b>{playerStats.from}</b> and <b>{playerStats.to}</b>,
    last seen <b>{playerStats.lastLeft}</b></span>
{:else if player === origPlayer && player !== ""}
  <span id="player-stats">No data found for <b>{player}</b></span>
{/if}<br>
<span>Rolling Averages</span>
<CheckboxButton bind:value={$rollingAverageSwitches[0]}>Raw</CheckboxButton>
<CheckboxButton bind:value={$rollingAverageSwitches[60]}>1h</CheckboxButton>
<CheckboxButton bind:value={$rollingAverageSwitches[720]}>12h</CheckboxButton>
<CheckboxButton bind:value={$rollingAverageSwitches[1440]}>1d</CheckboxButton>
<CheckboxButton bind:value={$rollingAverageSwitches[10080]}>7d</CheckboxButton>