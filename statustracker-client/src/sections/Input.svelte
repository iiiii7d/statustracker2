<script lang="ts">
  import moment from "moment";
  import { onMount } from "svelte";
  import CheckboxButton from "../comps/CheckboxButton.svelte";
  import { getLines } from "../retrieve-data";
  import { data, playerActiveTimes, rollingAverageSwitches, type RollingAverage } from "../stores";
  let origPlayer = "";

  let from = moment().subtract(1, "d").local(true).toISOString(true).slice(0, 16);
  let to = moment().add(1, "m").local(true).toISOString(true).slice(0, 16);
  let player = "";
  $: player = player.trim()
  onMount(query);
  rollingAverageSwitches.subscribe(query)

  async function query() {
    origPlayer = player;
    $data = await getLines(
      from !== "" ? Math.floor(moment(from).unix()/60) : 0,
      to !== "" ? Math.floor(moment(to).unix()/60) : 4294967295,
      player,
      Object.entries($rollingAverageSwitches).filter(([_, v]) => v)
      .map(([k, _]) => parseInt(k) as RollingAverage).sort()
    )
  }
  const version = __APP_VERSION__;

  function getPlayerStats(times: [moment.Moment, moment.Moment][]): any | null {
    if (times.length == 0) return null;
    let totalTime = times.map(([f, t]) => moment.duration(moment(t).diff(moment(f)))).reduce((acc, dur) => acc.add(dur)).humanize()
    let lastSession = times[times.length - 1];
    let lastLeft = lastSession === undefined ? "further back in time" : "on " + lastSession[1].toLocaleString();
    return {
      totalTime,
      from: from ? moment(from).toLocaleString() : "the start of StatusTracker",
      to: to ? moment(to).toLocaleString() : "the heat death of the universe",
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
{#if playerStats && player === origPlayer && player !== ""}
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