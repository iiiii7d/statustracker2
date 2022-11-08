<script lang="ts">
  import moment from "moment";
  import { retrievePlayerCounts } from "./retrieve-data";
  let from = "";
  let to = "";
  let player = "";

  function query() {
    retrievePlayerCounts(
      from !== "" ? Math.floor(moment(from).unix()/3600) : 0,
      to !== "" ? Math.floor(moment(to).unix()/3600) : 4294967295,
    )
  }
  const version = __APP_VERSION__;
</script>

<b style="font-size: 1.5em;">StatusTracker v{version}&nbsp;&nbsp;|&nbsp;&nbsp;</b>

<span>Show activity</span>
<label for="from">from </label><input type="datetime-local" id="from" bind:value={from}/>
<label for="to">to </label><input type="datetime-local" id="to" bind:value={to} />
<label for="player">for player </label><input type="text" id="player" bind:value={player} />
<button on:click={query}>Query</button>