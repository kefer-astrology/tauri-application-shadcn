<!-- Time Navigation Panel Component -->
<script lang="ts">
  import { 
    timeNavigation, 
    effectiveTime,
    stepForward, 
    stepBackward,
    type TimeStep
  } from '$lib/stores/timeNavigation.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import * as Select from '$lib/components/ui/select/index.js';
  import * as Popover from '$lib/components/ui/popover/index.js';
  import * as Calendar from '$lib/components/ui/calendar/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { CalendarDate, getLocalTimeZone, parseDate, today } from '@internationalized/date';
  import { formatTime } from '$lib/stores/timeNavigation.svelte';

  // Step amount options (1-30)
  const stepAmountOptions = Array.from({ length: 30 }, (_, i) => i + 1);
  
  // Step unit options (granularity)
  const stepUnitOptions: Array<{ value: TimeStep['unit']; label: string }> = [
    { value: 'seconds', label: t('time_nav_seconds', {}, 'Seconds') },
    { value: 'minutes', label: t('time_nav_minutes', {}, 'Minutes') },
    { value: 'hours', label: t('time_nav_hours', {}, 'Hours') },
    { value: 'days', label: t('time_nav_days', {}, 'Days') }
  ];

  // Current step state - use derived to access reactive values
  const nav = $derived(timeNavigation);
  let stepAmount = $state(String(timeNavigation.step.value));
  let stepUnit = $state(timeNavigation.step.unit);
  
  // Sync stepAmount and stepUnit when timeNavigation.step changes
  $effect(() => {
    stepAmount = String(timeNavigation.step.value);
    stepUnit = timeNavigation.step.unit;
  });

  // Update navigation step when selectors change
  $effect(() => {
    const amount = parseInt(stepAmount) || 1;
    timeNavigation.step = { unit: stepUnit, value: amount };
  });

  // Date picker state
  const effective = $derived(effectiveTime());
  const currentDate = $derived(() => {
    const d = effectiveTime();
    return new CalendarDate(d.getFullYear(), d.getMonth() + 1, d.getDate());
  });

  let selectedDate = $state<CalendarDate | undefined>(undefined);
  let datePopoverOpen = $state(false);
  const effectiveTimeValue = $derived(effectiveTime().getTime());
  let lastEffectiveTime = $state(0);
  
  // Initialize selectedDate and lastEffectiveTime
  $effect(() => {
    const currentTime = effectiveTimeValue;
    if (!selectedDate) {
      selectedDate = currentDate();
    }
    if (lastEffectiveTime === 0) {
      lastEffectiveTime = currentTime;
    }
  });

  // Sync selectedDate with currentDate when effective time changes externally (not from our updates)
  $effect(() => {
    const currentTime = effective.getTime();
    // Only update if the time changed externally (not from our own updates)
    if (currentTime !== lastEffectiveTime && !datePopoverOpen) {
      selectedDate = currentDate();
      lastEffectiveTime = currentTime;
    }
  });

  // Update date when selected and popover closes
  function handleDateChange(newDate: CalendarDate | undefined) {
    if (newDate) {
      const d = effective;
      const updatedDate = new Date(
        newDate.year,
        newDate.month - 1,
        newDate.day,
        d.getHours(),
        d.getMinutes(),
        d.getSeconds()
      );
      lastEffectiveTime = updatedDate.getTime();
      timeNavigation.currentTime = updatedDate;
    }
  }

  // Time input state
  let timeValue = $state('');
  let isTimeInputFocused = $state(false);
  let lastTimeValue = $state('');
  
  // Initialize and sync time value (only when not focused to avoid interrupting user input)
  $effect(() => {
    if (!isTimeInputFocused) {
      const d = effective;
      const hours = String(d.getHours()).padStart(2, '0');
      const minutes = String(d.getMinutes()).padStart(2, '0');
      const seconds = String(d.getSeconds()).padStart(2, '0');
      const newTimeValue = `${hours}:${minutes}:${seconds}`;
      if (newTimeValue !== lastTimeValue) {
        timeValue = newTimeValue;
        lastTimeValue = newTimeValue;
      }
    }
  });

  function updateTime(e: Event) {
    const input = e.target as HTMLInputElement;
    const parts = input.value.split(':');
    const hours = parseInt(parts[0]) || 0;
    const minutes = parseInt(parts[1]) || 0;
    const seconds = parseInt(parts[2]) || 0;
    const d = effective;
    const newDate = new Date(
      d.getFullYear(),
      d.getMonth(),
      d.getDate(),
      hours,
      minutes,
      seconds
    );
    lastEffectiveTime = newDate.getTime();
    timeNavigation.currentTime = newDate;
  }

  // Location state
  let locationValue = $state('');

  // Update button handler
  function handleUpdate() {
    // This will trigger any updates needed when the update button is clicked
    // The actual update logic will be handled by the timeNavigation store
    console.log('Update clicked', { date: selectedDate, time: timeValue, location: locationValue });
  }

  // Format date for display
  function formatDate(date: CalendarDate): string {
    const d = date.toDate(getLocalTimeZone());
    return d.toLocaleDateString('cs-CZ', { 
      day: '2-digit', 
      month: 'long', 
      year: 'numeric',
      weekday: 'long'
    });
  }
</script>

<div class="space-y-3 text-sm">
  <!-- Row 1: Left/right buttons, amount selector, granularity selector -->
  <div class="flex items-center gap-2">
    <button 
      type="button"
      class="px-2 py-1.5 text-xs border rounded hover:bg-muted/50 transition-colors flex-shrink-0"
      onclick={stepBackward}
      title={t('time_nav_previous', {}, 'Previous')}
    >
      ⏪
    </button>
    
    <Select.Root type="single" bind:value={stepAmount}>
      <Select.Trigger class="h-8 px-3 text-sm font-medium flex-1 min-w-[60px]">
        {#snippet children()}
          {stepAmount || '1'}
        {/snippet}
      </Select.Trigger>
      <Select.Content>
        <Select.Group>
          {#each stepAmountOptions as amount}
            <Select.Item value={String(amount)} label={String(amount)}>
              {amount}
            </Select.Item>
          {/each}
        </Select.Group>
      </Select.Content>
    </Select.Root>

    <Select.Root type="single" bind:value={stepUnit}>
      <Select.Trigger class="h-8 px-3 text-sm font-medium flex-1 min-w-[80px]">
        {#snippet children()}
          {stepUnitOptions.find(u => u.value === stepUnit)?.label || t('time_nav_hours', {}, 'Hours')}
        {/snippet}
      </Select.Trigger>
      <Select.Content>
        <Select.Group>
          {#each stepUnitOptions as unit}
            <Select.Item value={unit.value} label={unit.label}>
              {unit.label}
            </Select.Item>
          {/each}
        </Select.Group>
      </Select.Content>
    </Select.Root>

    <button 
      type="button"
      class="px-2 py-1.5 text-xs border rounded hover:bg-muted/50 transition-colors flex-shrink-0"
      onclick={stepForward}
      title={t('time_nav_next', {}, 'Next')}
    >
      ⏩
    </button>
  </div>

  <!-- Row 2: Date icon + date selector -->
  <div class="flex items-center gap-2">
    <div class="flex-shrink-0 w-5 h-5 flex items-center justify-center opacity-75">
      📅
    </div>
    <Popover.Root bind:open={datePopoverOpen}>
      <Popover.Trigger>
        <Button
          variant="outline"
          class="flex-1 h-8 px-2 text-xs justify-start font-normal"
        >
          {selectedDate ? formatDate(selectedDate) : formatDate(currentDate())}
        </Button>
      </Popover.Trigger>
      <Popover.Content class="w-auto p-0" align="start">
        <Calendar
          bind:value={selectedDate}
          locale="cs-CZ"
          onchange={(value) => {
            if (value) {
              handleDateChange(value);
            }
            datePopoverOpen = false;
          }}
        >
          {#snippet children({ months, weekdays })}
            <Calendar.Months>
              {#each months as month, monthIndex (month)}
                <Calendar.Month>
                  <Calendar.Header>
                    <Calendar.PrevButton />
                    <Calendar.Caption />
                    <Calendar.NextButton />
                  </Calendar.Header>
                  <Calendar.Grid>
                    <Calendar.GridHead>
                      <Calendar.GridRow>
                        {#each weekdays as weekday}
                          <Calendar.HeadCell>{weekday}</Calendar.HeadCell>
                        {/each}
                      </Calendar.GridRow>
                    </Calendar.GridHead>
                    <Calendar.GridBody>
                      <Calendar.GridRow let:week>
                        {#each week as date}
                          <Calendar.Cell>
                            <Calendar.Day {date} />
                          </Calendar.Cell>
                        {/each}
                      </Calendar.GridRow>
                    </Calendar.GridBody>
                  </Calendar.Grid>
                </Calendar.Month>
              {/each}
            </Calendar.Months>
          {/snippet}
        </Calendar>
      </Popover.Content>
    </Popover.Root>
  </div>

  <!-- Row 3: Time icon + time selector -->
  <div class="flex items-center gap-2">
    <div class="flex-shrink-0 w-5 h-5 flex items-center justify-center opacity-75">
      🕐
    </div>
    <Input
      type="text"
      class="flex-1 h-8 px-2 text-xs font-mono"
      bind:value={timeValue}
      onfocus={() => isTimeInputFocused = true}
      onblur={() => {
        isTimeInputFocused = false;
        updateTime(new Event('change'));
      }}
      onchange={updateTime}
      placeholder="HH:MM:SS"
      pattern="[0-9]{2}:[0-9]{2}:[0-9]{2}"
    />
  </div>

  <!-- Row 4: Location pin + location selector + update button -->
  <div class="flex items-center gap-2">
    <div class="flex-shrink-0 w-5 h-5 flex items-center justify-center opacity-75">
      📍
    </div>
    <Input
      type="text"
      class="flex-1 h-8 px-2 text-xs"
      bind:value={locationValue}
      placeholder={t('new_location', {}, 'Location')}
    />
    <Button
      type="button"
      class="h-8 px-3 text-xs flex-shrink-0"
      onclick={handleUpdate}
    >
      {t('time_nav_update', {}, 'Update')}
    </Button>
  </div>
</div>
