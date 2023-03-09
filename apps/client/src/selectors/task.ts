import { Api, TaskEntity } from "../../Api";
import { atom } from "recoil";

// API
export const api = new Api({
  baseUrl: "http://localhost:3000"
});

// Atoms
export const listAtom = atom<TaskEntity[]>({
  key: "todoList",
  default: await api.task
    .appControllerGetTaskList()
    .then((response) => {
      return response.data;
    })
    .catch((error) => {
      console.error(error);
      return [];
    })
});

export const selectedTaskIdAtom = atom({
  key: "selectedTaskIdState",
  default: ``
});

export const isModalOpenAtom = atom({
  key: "isModalOpenState",
  default: false
});

export const modalInputValueAtom = atom({
  key: "modalInputValueState",
  default: ``
});
