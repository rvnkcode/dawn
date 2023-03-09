import { TaskEntity } from "../../Api";
import { api, listAtom, isModalOpenAtom, modalInputValueAtom, selectedTaskIdAtom } from "../selectors/task";
import { DeleteFilled } from "@ant-design/icons";
import { Button, Checkbox, Empty, Layout, message } from "antd";
import { CheckboxChangeEvent } from "antd/es/checkbox";
import { MouseEvent } from "react";
import { useRecoilState, useSetRecoilState } from "recoil";

export default function ListComponent() {
  const [list, setList] = useRecoilState(listAtom);

  const deleteSelectedTask = (e: MouseEvent<HTMLAnchorElement> | MouseEvent<HTMLButtonElement>) => {
    api.task
      .appControllerDeleteTask((e.currentTarget as HTMLButtonElement).value)
      .then((response) => {
        setList(list.filter((task) => task.id !== response.data.id));
      })
      .catch((error) => {
        console.error(error);
        message.error(`Cannot remove selected task`);
      });
  };

  const toggleChecked = (e: CheckboxChangeEvent) => {
    api.task.appControllerUpdateTask(e.target.value, { isDone: e.target.checked }).catch((error) => {
      console.error(error);
      message.error(`Cannot complete task`);
    });
  };

  // Task edit modal
  const setIsModalOpen = useSetRecoilState(isModalOpenAtom);
  const setModalInputValue = useSetRecoilState(modalInputValueAtom);
  const setSelectedTaskIdState = useSetRecoilState(selectedTaskIdAtom);

  const openTaskEditModal = (e: MouseEvent<HTMLAnchorElement> | MouseEvent<HTMLButtonElement>) => {
    setModalInputValue(e.currentTarget.innerText);
    setSelectedTaskIdState((e.currentTarget as HTMLButtonElement).value);
    setIsModalOpen(true);
  };

  const todoListComponent = (list: Array<TaskEntity>) => {
    const listItems = list.map((task) => (
      <li key={task.id} style={{ display: "flex", justifyContent: "space-between" }}>
        <Checkbox defaultChecked={task.isDone} onChange={toggleChecked} value={task.id}>
          <Button type="text" size="small" onClick={(e) => openTaskEditModal(e)} value={task.id}>
            <span>{task.title}</span>
          </Button>
        </Checkbox>
        <Button type="text" size="small" value={task.id} onClick={(e) => deleteSelectedTask(e)}>
          <DeleteFilled />
        </Button>
      </li>
    ));

    return <ul>{listItems}</ul>;
  };

  return (
    <Content>
      {list.length > 0 ? (
        todoListComponent(list)
      ) : (
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="Your inbox is empty - time to celebrate!" />
      )}
    </Content>
  );
}
const { Content } = Layout;
