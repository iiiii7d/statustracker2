<script lang="ts">
  interface Props {
    value?: boolean;
    children?: import('svelte').Snippet;
  }

  let { value = $bindable(false), children }: Props = $props();
  let button: HTMLButtonElement | undefined = $state();

  function toggle() {
    value = !value;
    if (value) {
      button?.classList.add("checked");
    } else {
      button?.classList.remove("checked");
    }
  }
</script>
<style lang="scss">
  button.checked { 
    background-color: #8c0;
    color: #000;
    border: unset;
    box-shadow: #8c08 0px 3px;

    &:hover {
      background-color: #bf3;
      color: #000;
      box-shadow: #bf38 0px 4px;
    }
  }
</style>
<button onclick={toggle} bind:this={button} class:checked={value}>{@render children?.()}</button>