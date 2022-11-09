<script lang="ts">
  import moment from "moment";
  import { retrievePlayerCounts } from "./retrieve-data";
  import { playerActiveTimes } from "./stores";
  let origPlayer = "";

  let from = "";
  let to = "";
  let player = "";
  $: player = player.trim()

  function query() {
    origPlayer = player;
    retrievePlayerCounts(
      from !== "" ? Math.floor(moment(from).unix()/3600) : 0,
      to !== "" ? Math.floor(moment(to).unix()/3600) : 4294967295,
      player
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
      from: moment(from || 0).toLocaleString(),
      to: moment(to || 4294967295 * 3600).toLocaleString(),
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
</style>

<b style="font-size: 1.5em; color: #fc0">StatusTracker v{version}&nbsp;&nbsp;|&nbsp;&nbsp;</b>

<span>Show activity</span>
<label for="from">from </label><input type="datetime-local" id="from" bind:value={from}/>
<label for="to">to </label><input type="datetime-local" id="to" bind:value={to} />
<label for="player">for player </label><input type="text" id="player" bind:value={player} />
<button on:click={query}>Query</button><br>
{#if playerStats && player === origPlayer}
  <span id="player-stats"><b>{player}</b> has played
    for <b>{playerStats.totalTime}</b> between <b>{playerStats.from}</b> and <b>{playerStats.to}</b>,
    last log off was <b>{playerStats.lastLeft}</b></span>
{:else if player === origPlayer && player !== ""}
  <span id="player-stats">No data found for <b>{player}</b></span>
{/if}