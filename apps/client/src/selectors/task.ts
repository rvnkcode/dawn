import { Api, TaskEntity } from "../../Api";
import { atom } from "recoil";

// API
export const api = new Api({
  baseUrl: "http://localhost:3000"
});

// Atoms
export const listState = atom<TaskEntity[]>({
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

export const selectedTaskIdState = atom({
  key: "selectedTaskIdState",
  default: ``
});

export const isModalOpenState = atom({
  key: "isModalOpenState",
  default: false
});

export const modalInputValueState = atom({
  key: "modalInputValueState",
  default: ``
});
