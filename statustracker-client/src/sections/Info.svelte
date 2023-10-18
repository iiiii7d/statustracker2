<script lang="ts">
  import SvelteMarkdown from "svelte-markdown";
  import source from "../../../README.md";
  import source2 from "../../../changelog.md";
</script>

<SvelteMarkdown {source} />
<hr>
<SvelteMarkdown source={source2} />