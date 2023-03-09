import { api, listAtom } from "../selectors/task";
import { DeleteFilled } from "@ant-design/icons";
import { Button, Layout, message } from "antd";
import { useRecoilState } from "recoil";

const { Footer } = Layout;

export default function FooterComponent() {
  const [list, setList] = useRecoilState(listAtom);

  const clearList = () => {
    if (list.length > 0) {
      api.task
        .appControllerDeleteAllTask()
        .then(() => {
          setList([]);
        })
        .catch((error) => {
          console.error(error);
          message.error(`Cannot clear to-do list`);
        });
    }
  };

  return (
    <Footer style={{ position: "fixed", bottom: "0", left: "0", width: "100%", textAlign: "center" }}>
      {/* TODO: Add task counter and progress to footer */}
      <Button onClick={clearList} disabled={list.length == 0 ? true : false}>
        <DeleteFilled />
      </Button>
    </Footer>
  );
}
