import { invoke } from "@tauri-apps/api/core";

export interface Recording {
  id: string;
  timestamp: string;
  duration_seconds: number;
  status: string;
  title: string;
  description: string;
}

export interface Quest {
  task_id: string;
  title: string;
  original_instruction: string;
  concrete_scenario: string;
  objective: string;
  relevant_applications: string[];
  subgoals: string[];
}

export async function generateQuest(prompt: string, address: string): Promise<Quest> {
  // Get screenshot
  // const screenshot = await invoke('take_screenshot');
  
  // Get applications
  const apps = await invoke("list_apps", { includeIcons: false });
  const app_list = 
    ((apps as { name: any; path: any; }[])
      .map((app: { name: any; path: any; }) => 
        `${app.name}`
    )).join('\n')

  // Call quest endpoint
  const response = await fetch('http://localhost/api/gym/quest', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      // screenshot,
      installed_applications: app_list,
      address,
      prompt,
    }),
  });

  if (!response.ok) {
    throw new Error('Failed to generate quest');
  }

  return await response.json();
}

export async function startRecording() {
  try {
    await invoke('start_recording');
  } catch (error) {
    console.error('Failed to start recording:', error);
    throw error;
  }
}

export async function stopRecording() {
  try {
    await invoke('stop_recording');
  } catch (error) {
    console.error('Failed to stop recording:', error);
    throw error;
  }
}

export async function checkQuestProgress(quest: Quest): Promise<{completed_subgoals: boolean[], completed_objectives: number}> {
  try {
    // Get screenshots
    const screenshots = [];
    {
      const screenshot = await invoke('take_screenshot');
      screenshots.push(screenshot);
    }

    // Call progress endpoint
    const response = await fetch('http://localhost/api/gym/progress', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        quest,
        screenshots,
      }),
    });

    if (!response.ok) {
      throw new Error('Failed to check progress');
    }

    return await response.json();
  } catch (error) {
    console.error('Failed to check quest progress:', error);
    throw error;
  }
}
