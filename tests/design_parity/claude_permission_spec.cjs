// #931: explicit callback choice uses the existing advanced profile form.
const open="document.querySelector('.startup-profile .profile-tools button').click();document.getElementById('start-read-only-details').open=true";
module.exports=[
 {name:'claude-callback-choice',props:['fontSize','flexDirection','gap'],keys:['fontSize','flexDirection','gap'],pixel:false,mock:{run:"openProfileEditor('claude');profileField('advanced').open=true",selector:'#profile-callback-label'},app:{run:open,selector:'#start-claude-callback-label'}},
 {name:'claude-callback-checkbox',assert:"(()=>{const c=document.getElementById('start-claude-callback'),r=c.getBoundingClientRect();return c.checked&&r.width>0&&r.width<=18&&r.height>0&&r.height<=18;})()",app:{run:open}}
];
