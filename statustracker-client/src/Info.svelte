<script lang="ts">
  import SvelteMarkdown from "svelte-markdown";
  import source from "../../README.md?raw";
</script>

<SvelteMarkdown {source}></SvelteMarkdown>