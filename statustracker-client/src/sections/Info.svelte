<script lang="ts">
  import SvelteMarkdown from "svelte-markdown";
  import source from "../../../README.md?raw";
  import source2 from "../../../changelog.md?raw";
</script>

<SvelteMarkdown {source} />
<hr>
<SvelteMarkdown source={source2} />