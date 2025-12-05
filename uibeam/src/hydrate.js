const { default: init, hydraters } = await import('/.uibeam/hydrate.js');

await init();

document.querySelectorAll('[data-uibeam-hydrater]').forEach((container) => {
  const hydraterName = container.getAttribute('data-uibeam-hydrater');
  if (!hydraterName) {
    console.error(`[uibeam] no hydrater name: ${container}`);
    return;
  }
  const hydrater = hydraters[hydraterName];
  if (!hydrater) {
    console.error(`[uibeam] no hydrater found for name '${hydraterName}': ${container}`);
    return;
  }
  
  const propsStr = container.getAttribute('data-uibeam-props');
  if (!propsStr) {
    console.error(`[uibeam] no props string: ${container}`);
    return;
  }
  let props = null;
  try {
    props = JSON.parse(propsStr);
  } catch (e) {
    console.error(`[uibeam] failed to parse props JSON '${propsStr}': ${e}`);
    return;
  }
  if (!props) {
    console.error(`[uibeam] no props parsed: ${container}`);
    return;
  }
  
  try {
    hydrater(props, container);
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    console.error(`[uibeam] failed to hydrate with '${hydraterName}' and props '${JSON.stringify(propsStr)}': ${message}`);
  }
});
