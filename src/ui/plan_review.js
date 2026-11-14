// GitHub #686: the plan card's heading line, the ownership summary under the task rows and the detail strip of the selected task (mock step 3).
const acPlanReview = (() => {
  const overlap = (a, b) => a.some(x => b.some(y => x === y || x.startsWith(y.endsWith('/') ? y : y + '/') || y.startsWith(x.endsWith('/') ? x : x + '/')));
  const changes={added:'startup.replan_added',changed:'startup.replan_changed',removed:'startup.replan_removed'};
  const reasons={added:'startup.replan_reason_added',changed:'startup.replan_reason_changed',removed:'startup.replan_reason_removed'};
  const fields={order:'startup.replan_field_order',id:'startup.replan_field_id',agent:'startup.replan_field_agent',prompt:'startup.replan_field_prompt',paths:'startup.replan_field_paths',depends_on:'startup.replan_field_depends_on',test_command:'startup.replan_field_test_command',test_expect_failure:'startup.expected_test_failure',model:'startup.replan_field_model',fallback:'startup.replan_field_fallback',role:'startup.replan_field_role',execution_options:'startup.replan_field_execution_options',task_type:'startup.replan_field_task_type'};
  const conflicts = tasks => tasks.flatMap((t, i) => tasks.slice(i + 1).filter(o => overlap(t.paths || [], o.paths || [])).map(o => `${t.id} / ${o.id}`));
  function detail(task, title) {
    const paths = (task.paths || []).join(', ') || '—', deps = (task.depends_on || []).join(', ') || null, test = (task.test_expect_failure?acMessage('startup.expected_test_failure')+' · ':'')+((task.test_command || []).join(' ') || '—');
    return acMessage('startup.plan_detail', { value1: task.id, value2: title, value3: paths, value4: deps || acMessage('startup.plan_no_dependencies'), value5: test });
  }
  return {
    decorate(list, removed, replan, titleOf) {
      removed.replaceChildren();removed.hidden=true;
      if(!replan)return;
      for(const change of replan.changes||[]){
        let row=[...list.children].find(node=>node.querySelector('[data-key=id]')?.value===change.id);
        if(change.kind==='removed'){
          row=document.createElement('details');row.className='startup-task plan-removed';
          const summary=document.createElement('summary');summary.className='task-summary';
          const title=document.createElement('span');title.className='task-oneline';title.textContent=titleOf(change.task)||change.id;
          summary.append(title);row.append(summary);removed.append(row);removed.hidden=false;
        }
        if(!row)continue;row.dataset.change=change.kind;
        const tag=document.createElement('span');tag.className='plan-tag plan-change';tag.dataset.change=change.kind;
        AC_I18N.bind(tag,()=>acMessage(changes[change.kind]));
        row.querySelector('.task-oneline').append(tag);
        const reason=document.createElement('span');reason.className='plan-change-reason';
        AC_I18N.bind(reason,()=>acMessage(reasons[change.kind],{value1:(change.fields||[]).map(field=>fields[field]?acMessage(fields[field]):field).join(', ')}));
        row.querySelector('summary').append(reason);
        if(change.kind==='removed'){const detail=document.createElement('p');detail.className='task-prompt';detail.textContent=change.task.prompt;row.append(detail);}
      }
    },
    meaningfulBrief(spec) {
      const ids = new Set((spec.specs || []).map(task => task.id));
      const lines = String(spec.brief || "").split(/\r?\n/).map(line => line.trim()).filter(Boolean);
      return lines.some(line => !/^Tasks?:$/i.test(line) && !ids.has(line.replace(/^[-*]\s+/, "")));
    },
    // `box` holds the two lines; `select(index)` re-renders the strip when another row is opened.
    render(box, tasks, selected = 0, titleOf = task => task.id, phase = {phase: 1, phase_total: 1}) {
      box.replaceChildren();
      if (!tasks.length) { box.hidden = true; return; }
      box.hidden = false;
      const picked = tasks[Math.min(selected, tasks.length - 1)], title = titleOf(picked) || picked.id, clash = conflicts(tasks);
      const line = document.createElement('p'), strip = document.createElement('p');
      line.className = 'plan-position'; strip.className = 'plan-detail';
      AC_I18N.bind(line, () => acMessage('startup.plan_position', { ...phase, value1: Math.min(selected, tasks.length - 1) + 1, value2: tasks.length, value3: title, value4: clash.length ? acMessage('startup.plan_paths_overlap', { value1: clash.join(', ') }) : acMessage('startup.plan_paths_clear') }));
      AC_I18N.bind(strip, () => detail(picked, title));
      box.append(line, strip);
    },
  };
})();
