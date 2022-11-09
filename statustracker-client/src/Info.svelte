<script lang="ts">
  import SvelteMarkdown from "svelte-markdown";
  import source from "./info.md?raw";
</script>

<SvelteMarkdown {source}></SvelteMarkdown>